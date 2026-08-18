//! Native USB bulk link to the flashed display agent: handshake, RGB565 frame
//! streaming (full + dirty-tile), and incoming touch/ack draining.
//!
//! All USB I/O happens on a worker thread. The UI only swaps the "latest frame"
//! slot and reads status, so a stalled or unplugged device can never block the
//! egui update loop. The transport follows Markham's proven WBA65 design:
//! vendor-specific interface, 512-byte bulk IN/OUT endpoints, and `nusb` on
//! the host (no virtual serial port or baud-rate fiction).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565};
use embedded_gui_live::{Decoder, Msg, PROTO_VERSION};
use nusb::MaybeFuture;
use nusb::descriptors::TransferType;
use nusb::transfer::{Buffer, Bulk, Direction, In, Out, TransferError};

use crate::live_render::{RenderedFrame, changed_tiles};

/// Tile size used to partition dirty regions. 40x40 RGB565 = 3200 bytes, which
/// fits the device's advertised per-rectangle budget.
const TILE_W: u32 = 40;
const TILE_H: u32 = 40;

/// Bounds how long a single write may stall before we treat the device as gone.
const WRITE_TIMEOUT: Duration = Duration::from_millis(1500);
const READ_TIMEOUT: Duration = Duration::from_millis(20);
const AGENT_VID: u16 = 0x1209;
const AGENT_PID: u16 = 0xE611;
const USB_MPS: usize = 512;

/// A touch sample reported by the panel. Consumed by the optional touch-uplink
/// phase, which injects pointer events into the host `GuiContext`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TouchSample {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
}

/// Status shared between the UI thread and the link worker.
#[derive(Default)]
struct LinkState {
    handshaked: bool,
    alive: bool,
    error: Option<String>,
    tiles_sent: usize,
    touches: Vec<TouchSample>,
    /// Panel dimensions the agent advertised in `Ready`. Frames are fitted to
    /// this so rectangles never address pixels the panel does not have.
    fb_size: Option<(u16, u16)>,
}

struct Shared {
    /// Latest frame awaiting transmission. Newer frames replace older ones so a
    /// slow link coalesces instead of queueing stale work.
    latest: Mutex<Option<RenderedFrame>>,
    ready: Condvar,
    state: Mutex<LinkState>,
    quit: AtomicBool,
    /// Set when the next frame must repaint every tile rather than only the ones
    /// that differ, so the panel can be resynced after a drop or a manual push.
    force_full: AtomicBool,
}

/// A handle to a display-agent connection served by a background thread.
pub struct DeviceLink {
    shared: Arc<Shared>,
    device_id: String,
}

impl DeviceLink {
    /// Spawns a worker that opens `device_id` and begins serving frames. Returns
    /// immediately; connection failures surface through [`DeviceLink::error`].
    pub fn connect(device_id: &str) -> Result<Self, String> {
        let shared = Arc::new(Shared {
            latest: Mutex::new(None),
            ready: Condvar::new(),
            state: Mutex::new(LinkState {
                alive: true,
                ..Default::default()
            }),
            quit: AtomicBool::new(false),
            force_full: AtomicBool::new(false),
        });

        let worker_shared = Arc::clone(&shared);
        let name = device_id.to_string();
        thread::Builder::new()
            .name("studio-device-link".into())
            .spawn(move || worker(worker_shared, name))
            .map_err(|e| format!("Failed to spawn link thread: {e}"))?;

        Ok(Self {
            shared,
            device_id: device_id.to_string(),
        })
    }

    /// Stable identifier for the claimed USB device.
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Whether the device answered the handshake.
    pub fn is_handshaked(&self) -> bool {
        self.shared
            .state
            .lock()
            .map(|s| s.handshaked)
            .unwrap_or(false)
    }

    /// Panel size reported by the agent, once the handshake completes.
    pub fn framebuffer_size(&self) -> Option<(u16, u16)> {
        self.shared.state.lock().ok().and_then(|s| s.fb_size)
    }

    /// Whether the worker is still running.
    pub fn is_alive(&self) -> bool {
        self.shared.state.lock().map(|s| s.alive).unwrap_or(false)
    }

    /// Takes the most recent error, if any.
    pub fn take_error(&self) -> Option<String> {
        self.shared
            .state
            .lock()
            .ok()
            .and_then(|mut s| s.error.take())
    }

    /// Drains touch samples reported by the panel.
    #[allow(dead_code)]
    pub fn take_touches(&self) -> Vec<TouchSample> {
        self.shared
            .state
            .lock()
            .map(|mut s| std::mem::take(&mut s.touches))
            .unwrap_or_default()
    }

    /// Queues `frame` for transmission, replacing any not-yet-sent frame. Never
    /// blocks on device I/O.
    pub fn submit(&self, frame: RenderedFrame) {
        if let Ok(mut slot) = self.shared.latest.lock() {
            *slot = Some(frame);
            self.shared.ready.notify_one();
        }
    }

    /// Like [`DeviceLink::submit`], but repaints every tile instead of just the
    /// changed ones. Used by **Push Frame** so a panel that drifted out of sync
    /// with the host's diff baseline can always be recovered.
    pub fn submit_full(&self, frame: RenderedFrame) {
        self.shared.force_full.store(true, Ordering::Release);
        self.submit(frame);
    }
}

impl Drop for DeviceLink {
    fn drop(&mut self) {
        self.shared.quit.store(true, Ordering::Release);
        self.shared.ready.notify_all();
    }
}

/// Records a fatal error and marks the link dead.
fn fail(shared: &Shared, msg: String) {
    if let Ok(mut state) = shared.state.lock() {
        state.error = Some(msg);
        state.alive = false;
    }
}

struct BulkPort {
    ep_out: nusb::Endpoint<Bulk, Out>,
    ep_in: nusb::Endpoint<Bulk, In>,
}

fn worker(shared: Arc<Shared>, device_id: String) {
    let mut port = match open_bulk_device(&device_id) {
        Ok(port) => port,
        Err(e) => return fail(&shared, e),
    };

    let mut decoder: Box<Decoder<4096>> = Box::new(Decoder::new());
    let mut scratch: Vec<u8> = Vec::new();
    let mut prev: Option<RenderedFrame> = None;
    let mut seq: u32 = 0;

    // Handshake.
    let hello = Msg::Hello {
        proto: PROTO_VERSION,
        fb_w: 0,
        fb_h: 0,
    };
    if let Err(e) = write_msg(&mut port, &mut scratch, &hello) {
        return fail(&shared, format!("write Hello: {e}"));
    }

    let deadline = Instant::now() + Duration::from_millis(750);
    while Instant::now() < deadline {
        if !drain(&mut port, &mut decoder, &shared) {
            break;
        }
        if shared.state.lock().map(|s| s.handshaked).unwrap_or(false) {
            break;
        }
    }

    loop {
        if shared.quit.load(Ordering::Acquire) {
            return;
        }

        // Wait for a frame to send.
        let frame = {
            let Ok(mut slot) = shared.latest.lock() else {
                return;
            };
            while slot.is_none() {
                if shared.quit.load(Ordering::Acquire) {
                    return;
                }
                let Ok((next, _)) = shared.ready.wait_timeout(slot, Duration::from_millis(200))
                else {
                    return;
                };
                slot = next;
                if slot.is_none() {
                    break;
                }
            }
            slot.take()
        };

        // Service incoming messages whether or not a frame is pending.
        if !drain(&mut port, &mut decoder, &shared) {
            return;
        }

        let Some(frame) = frame else { continue };

        seq = seq.wrapping_add(1);
        let full = shared.force_full.swap(false, Ordering::AcqRel)
            || match &prev {
                Some(p) => p.width != frame.width || p.height != frame.height,
                None => true,
            };
        let rects = if full {
            tile_cover(frame.width as u32, frame.height as u32, TILE_W, TILE_H)
        } else {
            changed_tiles(prev.as_ref().unwrap(), &frame, TILE_W, TILE_H)
        };

        if let Err(e) = send_frame(&mut port, &mut scratch, seq, full, &frame, &rects) {
            return fail(&shared, e);
        }

        if let Ok(mut state) = shared.state.lock() {
            state.tiles_sent = rects.len();
        }
        prev = Some(frame);
    }
}

/// Sends one frame as FrameBegin / FrameRect* / FrameEnd.
fn send_frame(
    port: &mut BulkPort,
    scratch: &mut Vec<u8>,
    seq: u32,
    full: bool,
    frame: &RenderedFrame,
    rects: &[(u16, u16, u16, u16)],
) -> Result<(), String> {
    write_msg(port, scratch, &Msg::FrameBegin { seq, full })?;

    let mut pixels: Vec<u8> = Vec::new();
    for (x, y, w, h) in rects {
        pixels.clear();
        pixels.reserve(*w as usize * *h as usize * 2);
        for row in 0..*h as u32 {
            let sy = *y as u32 + row;
            for col in 0..*w as u32 {
                let sx = *x as u32 + col;
                let px: Rgb565 = frame.pixels[(sy * frame.width as u32 + sx) as usize];
                pixels.extend_from_slice(&px.into_storage().to_le_bytes());
            }
        }
        write_msg(
            port,
            scratch,
            &Msg::FrameRect {
                seq,
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                pixels: &pixels,
            },
        )?;
    }

    write_msg(port, scratch, &Msg::FrameEnd { seq })?;
    Ok(())
}

/// Encodes and writes a message, bounding how long a stalled device may block.
fn write_msg(port: &mut BulkPort, scratch: &mut Vec<u8>, msg: &Msg<'_>) -> Result<(), String> {
    let need = msg.encoded_len();
    if scratch.len() < need {
        scratch.resize(need, 0);
    }
    let n = msg.encode(scratch).map_err(|e| format!("encode: {e:?}"))?;

    for chunk in scratch[..n].chunks(USB_MPS) {
        port.ep_out
            .transfer_blocking(chunk.to_vec().into(), WRITE_TIMEOUT)
            .into_result()
            .map_err(|e| format!("USB bulk OUT failed: {e:?}"))?;
    }
    Ok(())
}

/// Reads any pending bytes and applies decoded messages. Returns `false` if the
/// port failed fatally.
fn drain(port: &mut BulkPort, decoder: &mut Decoder<4096>, shared: &Shared) -> bool {
    let mut rx = [0u8; 512];
    port.ep_in.submit(Buffer::new(USB_MPS));
    match port
        .ep_in
        .wait_next_complete(READ_TIMEOUT)
        .map(|completion| completion.into_result())
    {
        Some(Ok(data)) if !data.is_empty() => {
            let count = data.len().min(rx.len());
            rx[..count].copy_from_slice(&data[..count]);
            let mut ready = None;
            let mut touches = Vec::new();
            decoder.feed(
                &rx[..count],
                |msg| match msg {
                    Msg::Ready { fb_w, fb_h, .. } => ready = Some((fb_w, fb_h)),
                    Msg::Touch { x, y, pressed } => touches.push(TouchSample { x, y, pressed }),
                    _ => {}
                },
                |_err| {},
            );
            if let Ok(mut state) = shared.state.lock() {
                if let Some((fb_w, fb_h)) = ready {
                    state.handshaked = true;
                    if fb_w > 0 && fb_h > 0 {
                        state.fb_size = Some((fb_w, fb_h));
                    }
                }
                state.touches.extend(touches);
            }
            true
        }
        Some(Ok(_)) | None => true,
        Some(Err(TransferError::Stall)) => true,
        Some(Err(e)) => {
            fail(shared, format!("USB bulk IN failed: {e:?}"));
            false
        }
    }
}

/// Full-cover tiling of a `w` x `h` frame.
fn tile_cover(w: u32, h: u32, tile_w: u32, tile_h: u32) -> Vec<(u16, u16, u16, u16)> {
    let mut rects = Vec::new();
    let mut ty = 0;
    while ty < h {
        let th = tile_h.min(h - ty);
        let mut tx = 0;
        while tx < w {
            let tw = tile_w.min(w - tx);
            rects.push((tx as u16, ty as u16, tw as u16, th as u16));
            tx += tile_w;
        }
        ty += tile_h;
    }
    rects
}

fn open_bulk_device(device_id: &str) -> Result<BulkPort, String> {
    let devices = nusb::list_devices()
        .wait()
        .map_err(|e| format!("USB enumeration failed: {e}"))?;
    let mut matches: Vec<_> = devices
        .filter(|d| {
            d.vendor_id() == AGENT_VID
                && d.product_id() == AGENT_PID
                && device_identifier(d) == device_id
        })
        .collect();

    let info = matches
        .pop()
        .ok_or_else(|| format!("Studio agent {device_id} is no longer connected"))?;
    let device = info
        .open()
        .wait()
        .map_err(|e| format!("Failed to open USB device: {e}"))?;
    let interface = device
        .claim_interface(0)
        .wait()
        .map_err(|e| format!("Failed to claim USB interface 0: {e}"))?;

    let mut ep_out_addr = None;
    let mut ep_in_addr = None;
    'outer: for alt in interface.descriptors() {
        for ep in alt.endpoints() {
            match (ep.transfer_type(), ep.direction()) {
                (TransferType::Bulk, Direction::Out) if ep_out_addr.is_none() => {
                    ep_out_addr = Some(ep.address())
                }
                (TransferType::Bulk, Direction::In) if ep_in_addr.is_none() => {
                    ep_in_addr = Some(ep.address())
                }
                _ => {}
            }
            if ep_out_addr.is_some() && ep_in_addr.is_some() {
                break 'outer;
            }
        }
    }

    let ep_out = interface
        .endpoint::<Bulk, Out>(ep_out_addr.ok_or("Missing bulk OUT endpoint")?)
        .map_err(|e| format!("Failed to open bulk OUT endpoint: {e}"))?;
    let ep_in = interface
        .endpoint::<Bulk, In>(ep_in_addr.ok_or("Missing bulk IN endpoint")?)
        .map_err(|e| format!("Failed to open bulk IN endpoint: {e}"))?;
    Ok(BulkPort { ep_out, ep_in })
}

fn device_identifier(info: &nusb::DeviceInfo) -> String {
    info.serial_number()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}:{}", info.bus_id(), info.device_address()))
}

/// Lists native USB bulk display agents for the UI.
pub fn list_devices() -> Vec<String> {
    nusb::list_devices()
        .wait()
        .map(|devices| {
            devices
                .filter(|d| d.vendor_id() == AGENT_VID && d.product_id() == AGENT_PID)
                .map(|d| device_identifier(&d))
                .collect()
        })
        .unwrap_or_default()
}
