//! Live streaming protocol between `embedded-gui-studio` (host) and a flashed
//! display agent running on the microcontroller.
//!
//! The host renders an `embedded-gui` screen into an RGB565 buffer, diffs it
//! against the previously sent frame, and pushes the changed rectangles to the
//! board over a byte stream (USB-CDC on the STM32WBA65 USB-HS device port, or a
//! UART fallback). The board blits those rectangles straight to the panel, so no
//! reflash is needed per design edit.
//!
//! # Why a hand-rolled binary codec
//!
//! Frame rectangles carry raw pixel payloads (up to tens of KiB). A
//! self-describing format such as postcard/COBS would either expand the payload
//! (COBS stuffing) or force the device to buffer whole frames before decoding.
//! Instead this module uses a compact, length-prefixed framing with a resync
//! magic and a CRC-16, which lets a constant-memory [`Decoder`] on the MCU
//! accept partial reads and stream bounded rectangles.
//!
//! # Wire format
//!
//! ```text
//! +--------+--------+------+-----------+------------------+--------+
//! | 0xE6   | 0x71   | type | len (u32) | payload (len B)  | crc16  |
//! +--------+--------+------+-----------+------------------+--------+
//!   magic0   magic1   u8     LE          message body       LE
//! ```
//!
//! The CRC-16 (CCITT-FALSE) covers `type`, the 4 length bytes, and the payload.
//! All multi-byte integers are little-endian. Pixels are RGB565, little-endian.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

/// Protocol version. Bump on any wire-incompatible change; both sides compare
/// this in the [`Msg::Hello`] / [`Msg::Ready`] handshake.
pub const PROTO_VERSION: u16 = 1;

const MAGIC0: u8 = 0xE6;
const MAGIC1: u8 = 0x71;

/// Bytes preceding every payload: `magic0, magic1, type, len(4)`.
pub const HEADER_LEN: usize = 7;
/// Trailing CRC-16 bytes.
pub const TRAILER_LEN: usize = 2;
/// Total framing overhead around a payload.
pub const FRAME_OVERHEAD: usize = HEADER_LEN + TRAILER_LEN;

// ── Message type tags ──────────────────────────────────────────────────────
const T_HELLO: u8 = 0x01;
const T_FRAME_BEGIN: u8 = 0x02;
const T_FRAME_RECT: u8 = 0x03;
const T_FRAME_END: u8 = 0x04;
const T_PING: u8 = 0x05;

const T_READY: u8 = 0x81;
const T_ACK: u8 = 0x82;
const T_NACK: u8 = 0x83;
const T_TOUCH: u8 = 0x84;
const T_PONG: u8 = 0x85;

/// Fixed-field header size of a [`Msg::FrameRect`] payload (before pixels).
pub const FRAME_RECT_HEADER: usize = 4 + 2 + 2 + 2 + 2; // seq + x + y + w + h

/// A decoded protocol message. Pixel data in [`Msg::FrameRect`] borrows from the
/// decoder (or caller) buffer to stay allocation-free on the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg<'a> {
    // Host -> Device
    /// Host handshake announcing protocol version and its render dimensions.
    Hello { proto: u16, fb_w: u16, fb_h: u16 },
    /// Marks the start of a frame. `full` means a complete repaint follows.
    FrameBegin { seq: u32, full: bool },
    /// One changed rectangle of RGB565 (LE) pixels. `pixels.len() == w*h*2`.
    FrameRect {
        seq: u32,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        pixels: &'a [u8],
    },
    /// Marks the end of a frame; device may present/latch here.
    FrameEnd { seq: u32 },
    /// Keepalive request.
    Ping,

    // Device -> Host
    /// Device handshake reply. `max_rect_bytes` bounds a single [`Msg::FrameRect`]
    /// pixel payload so the host can split large regions into bands.
    Ready {
        proto: u16,
        fb_w: u16,
        fb_h: u16,
        max_rect_bytes: u32,
    },
    /// Positive acknowledgement of `seq`.
    Ack { seq: u32 },
    /// Negative acknowledgement of `seq` with a [`NackCode`].
    Nack { seq: u32, code: u16 },
    /// Touch report from the panel back to the host.
    Touch { x: u16, y: u16, pressed: bool },
    /// Keepalive reply.
    Pong,
}

/// Well-known [`Msg::Nack`] codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum NackCode {
    /// Rectangle exceeded the device's `max_rect_bytes` budget.
    RectTooLarge = 1,
    /// Rectangle fell outside the panel bounds.
    OutOfBounds = 2,
    /// Protocol version mismatch.
    BadProto = 3,
    /// Malformed message body.
    Malformed = 4,
}

/// Errors produced while encoding a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// Output slice was too small for the encoded frame.
    BufferTooSmall,
    /// `FrameRect` pixel length did not equal `w * h * 2`.
    PixelLenMismatch,
}

/// Errors produced while decoding a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// Assembled payload failed CRC verification (stream resynced).
    BadCrc,
    /// Frame payload exceeded the decoder capacity (stream resynced).
    Overflow,
    /// Unknown message type tag (stream resynced).
    UnknownType(u8),
    /// Payload length did not match the message type's fixed fields.
    BadLength,
}

impl<'a> Msg<'a> {
    fn type_tag(&self) -> u8 {
        match self {
            Msg::Hello { .. } => T_HELLO,
            Msg::FrameBegin { .. } => T_FRAME_BEGIN,
            Msg::FrameRect { .. } => T_FRAME_RECT,
            Msg::FrameEnd { .. } => T_FRAME_END,
            Msg::Ping => T_PING,
            Msg::Ready { .. } => T_READY,
            Msg::Ack { .. } => T_ACK,
            Msg::Nack { .. } => T_NACK,
            Msg::Touch { .. } => T_TOUCH,
            Msg::Pong => T_PONG,
        }
    }

    /// Length of the payload (message body) this message encodes to.
    pub fn payload_len(&self) -> usize {
        match self {
            Msg::Hello { .. } => 6,
            Msg::FrameBegin { .. } => 5,
            Msg::FrameRect { pixels, .. } => FRAME_RECT_HEADER + pixels.len(),
            Msg::FrameEnd { .. } => 4,
            Msg::Ping => 0,
            Msg::Ready { .. } => 10,
            Msg::Ack { .. } => 4,
            Msg::Nack { .. } => 6,
            Msg::Touch { .. } => 5,
            Msg::Pong => 0,
        }
    }

    /// Total bytes this message occupies on the wire including framing.
    pub fn encoded_len(&self) -> usize {
        FRAME_OVERHEAD + self.payload_len()
    }

    /// Encodes this message into `out`, returning the number of bytes written.
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
        // Validate FrameRect pixel length up front.
        if let Msg::FrameRect { w, h, pixels, .. } = self {
            if pixels.len() != (*w as usize) * (*h as usize) * 2 {
                return Err(EncodeError::PixelLenMismatch);
            }
        }

        let payload_len = self.payload_len();
        let total = FRAME_OVERHEAD + payload_len;
        if out.len() < total {
            return Err(EncodeError::BufferTooSmall);
        }

        out[0] = MAGIC0;
        out[1] = MAGIC1;
        out[2] = self.type_tag();
        out[3..7].copy_from_slice(&(payload_len as u32).to_le_bytes());

        let body = &mut out[HEADER_LEN..HEADER_LEN + payload_len];
        self.encode_body(body);

        let crc = crc16(&out[2..HEADER_LEN + payload_len]);
        out[HEADER_LEN + payload_len..total].copy_from_slice(&crc.to_le_bytes());
        Ok(total)
    }

    fn encode_body(&self, body: &mut [u8]) {
        match *self {
            Msg::Hello { proto, fb_w, fb_h } => {
                body[0..2].copy_from_slice(&proto.to_le_bytes());
                body[2..4].copy_from_slice(&fb_w.to_le_bytes());
                body[4..6].copy_from_slice(&fb_h.to_le_bytes());
            }
            Msg::FrameBegin { seq, full } => {
                body[0..4].copy_from_slice(&seq.to_le_bytes());
                body[4] = full as u8;
            }
            Msg::FrameRect {
                seq,
                x,
                y,
                w,
                h,
                pixels,
            } => {
                body[0..4].copy_from_slice(&seq.to_le_bytes());
                body[4..6].copy_from_slice(&x.to_le_bytes());
                body[6..8].copy_from_slice(&y.to_le_bytes());
                body[8..10].copy_from_slice(&w.to_le_bytes());
                body[10..12].copy_from_slice(&h.to_le_bytes());
                body[FRAME_RECT_HEADER..FRAME_RECT_HEADER + pixels.len()].copy_from_slice(pixels);
            }
            Msg::FrameEnd { seq } => {
                body[0..4].copy_from_slice(&seq.to_le_bytes());
            }
            Msg::Ping | Msg::Pong => {}
            Msg::Ready {
                proto,
                fb_w,
                fb_h,
                max_rect_bytes,
            } => {
                body[0..2].copy_from_slice(&proto.to_le_bytes());
                body[2..4].copy_from_slice(&fb_w.to_le_bytes());
                body[4..6].copy_from_slice(&fb_h.to_le_bytes());
                body[6..10].copy_from_slice(&max_rect_bytes.to_le_bytes());
            }
            Msg::Ack { seq } => {
                body[0..4].copy_from_slice(&seq.to_le_bytes());
            }
            Msg::Nack { seq, code } => {
                body[0..4].copy_from_slice(&seq.to_le_bytes());
                body[4..6].copy_from_slice(&code.to_le_bytes());
            }
            Msg::Touch { x, y, pressed } => {
                body[0..2].copy_from_slice(&x.to_le_bytes());
                body[2..4].copy_from_slice(&y.to_le_bytes());
                body[4] = pressed as u8;
            }
        }
    }
}

/// Parses a message from a validated payload of the given type tag.
fn parse(msg_type: u8, body: &[u8]) -> Result<Msg<'_>, DecodeError> {
    let u16le = |b: &[u8]| u16::from_le_bytes([b[0], b[1]]);
    let u32le = |b: &[u8]| u32::from_le_bytes([b[0], b[1], b[2], b[3]]);

    match msg_type {
        T_HELLO => {
            if body.len() != 6 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Hello {
                proto: u16le(&body[0..2]),
                fb_w: u16le(&body[2..4]),
                fb_h: u16le(&body[4..6]),
            })
        }
        T_FRAME_BEGIN => {
            if body.len() != 5 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::FrameBegin {
                seq: u32le(&body[0..4]),
                full: body[4] != 0,
            })
        }
        T_FRAME_RECT => {
            if body.len() < FRAME_RECT_HEADER {
                return Err(DecodeError::BadLength);
            }
            let w = u16le(&body[8..10]);
            let h = u16le(&body[10..12]);
            let pixels = &body[FRAME_RECT_HEADER..];
            if pixels.len() != (w as usize) * (h as usize) * 2 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::FrameRect {
                seq: u32le(&body[0..4]),
                x: u16le(&body[4..6]),
                y: u16le(&body[6..8]),
                w,
                h,
                pixels,
            })
        }
        T_FRAME_END => {
            if body.len() != 4 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::FrameEnd {
                seq: u32le(&body[0..4]),
            })
        }
        T_PING => {
            if !body.is_empty() {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Ping)
        }
        T_READY => {
            if body.len() != 10 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Ready {
                proto: u16le(&body[0..2]),
                fb_w: u16le(&body[2..4]),
                fb_h: u16le(&body[4..6]),
                max_rect_bytes: u32le(&body[6..10]),
            })
        }
        T_ACK => {
            if body.len() != 4 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Ack {
                seq: u32le(&body[0..4]),
            })
        }
        T_NACK => {
            if body.len() != 6 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Nack {
                seq: u32le(&body[0..4]),
                code: u16le(&body[4..6]),
            })
        }
        T_TOUCH => {
            if body.len() != 5 {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Touch {
                x: u16le(&body[0..2]),
                y: u16le(&body[2..4]),
                pressed: body[4] != 0,
            })
        }
        T_PONG => {
            if !body.is_empty() {
                return Err(DecodeError::BadLength);
            }
            Ok(Msg::Pong)
        }
        other => Err(DecodeError::UnknownType(other)),
    }
}

// ── Streaming decoder ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Magic0,
    Magic1,
    Type,
    Len(u8),
    Payload,
    Crc(u8),
}

/// A constant-memory, resynchronizing frame decoder.
///
/// `CAP` is the maximum payload (message body) the decoder can assemble; size it
/// to `FRAME_RECT_HEADER + max_rect_bytes` on the device. Feed bytes via
/// [`Decoder::push`]; when it returns `Ok(true)` a complete frame is available
/// via [`Decoder::message`].
pub struct Decoder<const CAP: usize> {
    state: State,
    msg_type: u8,
    payload_len: u32,
    buf: [u8; CAP],
    got: usize,
    crc_lo: u8,
    ready: bool,
}

impl<const CAP: usize> Default for Decoder<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> Decoder<CAP> {
    /// Creates an empty decoder.
    pub const fn new() -> Self {
        Self {
            state: State::Magic0,
            msg_type: 0,
            payload_len: 0,
            buf: [0u8; CAP],
            got: 0,
            crc_lo: 0,
            ready: false,
        }
    }

    /// Resets to hunting for the next frame magic.
    fn resync(&mut self) {
        self.state = State::Magic0;
        self.got = 0;
        self.ready = false;
    }

    /// Feeds a single byte. Returns `Ok(true)` when a full, CRC-valid frame has
    /// been assembled (read it with [`Decoder::message`] before the next push).
    /// Returns `Err` on a framing/CRC/overflow fault, having already resynced.
    pub fn push(&mut self, byte: u8) -> Result<bool, DecodeError> {
        // A previously-ready frame is consumed as soon as we take another byte.
        if self.ready {
            self.resync();
        }

        match self.state {
            State::Magic0 => {
                if byte == MAGIC0 {
                    self.state = State::Magic1;
                }
            }
            State::Magic1 => {
                if byte == MAGIC1 {
                    self.state = State::Type;
                } else if byte == MAGIC0 {
                    self.state = State::Magic1;
                } else {
                    self.state = State::Magic0;
                }
            }
            State::Type => {
                self.msg_type = byte;
                self.payload_len = 0;
                self.state = State::Len(0);
            }
            State::Len(i) => {
                self.payload_len |= (byte as u32) << (8 * i as u32);
                if i == 3 {
                    let len = self.payload_len as usize;
                    if len > CAP {
                        self.resync();
                        return Err(DecodeError::Overflow);
                    }
                    self.got = 0;
                    self.state = if len == 0 {
                        State::Crc(0)
                    } else {
                        State::Payload
                    };
                } else {
                    self.state = State::Len(i + 1);
                }
            }
            State::Payload => {
                self.buf[self.got] = byte;
                self.got += 1;
                if self.got == self.payload_len as usize {
                    self.state = State::Crc(0);
                }
            }
            State::Crc(0) => {
                self.crc_lo = byte;
                self.state = State::Crc(1);
            }
            State::Crc(_) => {
                let got_crc = u16::from_le_bytes([self.crc_lo, byte]);
                let want = self.frame_crc();
                if got_crc == want {
                    self.ready = true;
                    return Ok(true);
                } else {
                    self.resync();
                    return Err(DecodeError::BadCrc);
                }
            }
        }
        Ok(false)
    }

    /// Feeds a slice, invoking `on_msg` for each complete frame. Decode faults
    /// are reported to `on_err` and do not abort the loop (the stream resyncs).
    pub fn feed<F, E>(&mut self, data: &[u8], mut on_msg: F, mut on_err: E)
    where
        F: FnMut(Msg<'_>),
        E: FnMut(DecodeError),
    {
        for &b in data {
            match self.push(b) {
                Ok(true) => match self.message() {
                    Ok(msg) => on_msg(msg),
                    Err(e) => on_err(e),
                },
                Ok(false) => {}
                Err(e) => on_err(e),
            }
        }
    }

    /// CRC over `type`, the 4 length bytes, and the assembled payload.
    fn frame_crc(&self) -> u16 {
        let mut crc = CRC_INIT;
        crc = crc16_step(crc, self.msg_type);
        for b in self.payload_len.to_le_bytes() {
            crc = crc16_step(crc, b);
        }
        for &b in &self.buf[..self.payload_len as usize] {
            crc = crc16_step(crc, b);
        }
        crc
    }

    /// Parses the frame assembled by the last successful [`Decoder::push`].
    pub fn message(&self) -> Result<Msg<'_>, DecodeError> {
        parse(self.msg_type, &self.buf[..self.payload_len as usize])
    }
}

// ── CRC-16/CCITT-FALSE ──────────────────────────────────────────────────────

const CRC_INIT: u16 = 0xFFFF;

#[inline]
fn crc16_step(mut crc: u16, byte: u8) -> u16 {
    crc ^= (byte as u16) << 8;
    let mut i = 0;
    while i < 8 {
        if crc & 0x8000 != 0 {
            crc = (crc << 1) ^ 0x1021;
        } else {
            crc <<= 1;
        }
        i += 1;
    }
    crc
}

/// Computes the CRC-16/CCITT-FALSE over `data`.
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc = CRC_INIT;
    for &b in data {
        crc = crc16_step(crc, b);
    }
    crc
}

// ── RGB565 helpers ──────────────────────────────────────────────────────────

/// Packs 8-bit R/G/B into RGB565.
#[inline]
pub fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3)
}

/// Writes an RGB565 value to a 2-byte little-endian slot.
#[inline]
pub fn put_rgb565_le(out: &mut [u8], value: u16) {
    out[0..2].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    fn roundtrip(msg: Msg<'_>) {
        let mut buf = vec![0u8; msg.encoded_len()];
        let n = msg.encode(&mut buf).unwrap();
        assert_eq!(n, msg.encoded_len());

        let mut dec = Decoder::<65536>::new();
        let mut got = None;
        for (i, &b) in buf.iter().enumerate() {
            let ready = dec.push(b).unwrap();
            if i + 1 == buf.len() {
                assert!(ready, "final byte should complete the frame");
            }
            if ready {
                got = Some(dec.message().unwrap().to_owned_like());
            }
        }
        assert_eq!(got.unwrap(), msg.to_owned_like());
    }

    // Test helper: clone into an owned representation independent of buffers.
    impl<'a> Msg<'a> {
        fn to_owned_like(&self) -> OwnedMsg {
            match *self {
                Msg::FrameRect {
                    seq,
                    x,
                    y,
                    w,
                    h,
                    pixels,
                } => OwnedMsg::FrameRect {
                    seq,
                    x,
                    y,
                    w,
                    h,
                    pixels: pixels.to_vec(),
                },
                ref other => OwnedMsg::Other(std::format!("{:?}", other)),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum OwnedMsg {
        FrameRect {
            seq: u32,
            x: u16,
            y: u16,
            w: u16,
            h: u16,
            pixels: Vec<u8>,
        },
        Other(std::string::String),
    }

    #[test]
    fn roundtrip_control_messages() {
        roundtrip(Msg::Hello {
            proto: PROTO_VERSION,
            fb_w: 320,
            fb_h: 240,
        });
        roundtrip(Msg::FrameBegin { seq: 7, full: true });
        roundtrip(Msg::FrameEnd { seq: 7 });
        roundtrip(Msg::Ping);
        roundtrip(Msg::Pong);
        roundtrip(Msg::Ready {
            proto: PROTO_VERSION,
            fb_w: 320,
            fb_h: 240,
            max_rect_bytes: 16384,
        });
        roundtrip(Msg::Ack { seq: 42 });
        roundtrip(Msg::Nack {
            seq: 42,
            code: NackCode::RectTooLarge as u16,
        });
        roundtrip(Msg::Touch {
            x: 100,
            y: 200,
            pressed: true,
        });
    }

    #[test]
    fn roundtrip_frame_rect() {
        let mut pixels = vec![0u8; 4 * 3 * 2];
        for (i, chunk) in pixels.chunks_mut(2).enumerate() {
            put_rgb565_le(chunk, rgb565(i as u8 * 10, i as u8 * 5, i as u8));
        }
        roundtrip(Msg::FrameRect {
            seq: 3,
            x: 8,
            y: 16,
            w: 4,
            h: 3,
            pixels: &pixels,
        });
    }

    #[test]
    fn pixel_len_mismatch_is_rejected() {
        let pixels = vec![0u8; 10];
        let msg = Msg::FrameRect {
            seq: 0,
            x: 0,
            y: 0,
            w: 4,
            h: 3,
            pixels: &pixels,
        };
        let mut buf = vec![0u8; 256];
        assert_eq!(msg.encode(&mut buf), Err(EncodeError::PixelLenMismatch));
    }

    #[test]
    fn decoder_resyncs_after_garbage() {
        let msg = Msg::Ack { seq: 99 };
        let mut buf = vec![0u8; msg.encoded_len()];
        msg.encode(&mut buf).unwrap();

        let mut dec = Decoder::<256>::new();
        // Leading noise then a valid frame.
        let mut stream: Vec<u8> = vec![0x00, 0xFF, 0xE6, 0x12, 0x34];
        stream.extend_from_slice(&buf);

        let mut acked = None;
        dec.feed(
            &stream,
            |m| {
                if let Msg::Ack { seq } = m {
                    acked = Some(seq);
                }
            },
            |_e| {},
        );
        assert_eq!(acked, Some(99));
    }

    #[test]
    fn decoder_reports_bad_crc() {
        let msg = Msg::Ack { seq: 1 };
        let mut buf = vec![0u8; msg.encoded_len()];
        msg.encode(&mut buf).unwrap();
        // Corrupt the payload after the header.
        buf[HEADER_LEN] ^= 0xFF;

        let mut dec = Decoder::<256>::new();
        let mut errors = 0;
        dec.feed(&buf, |_m| panic!("should not decode"), |_e| errors += 1);
        assert!(errors >= 1);
    }

    #[test]
    fn decoder_overflow_when_payload_exceeds_capacity() {
        // Announce a payload larger than CAP and ensure we get Overflow + resync.
        let mut dec = Decoder::<8>::new();
        let mut got_overflow = false;
        let header = [MAGIC0, MAGIC1, T_FRAME_RECT, 0x00, 0x01, 0x00, 0x00]; // len = 256
        for &b in &header {
            if let Err(DecodeError::Overflow) = dec.push(b) {
                got_overflow = true;
            }
        }
        assert!(got_overflow);
    }
}
