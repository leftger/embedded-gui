//! Optional Embassy integration helpers (feature `embassy`).
//!
//! Core present APIs work without Embassy; enable this module for
//! [`EmbassyWaitTransfer`] and [`FrameClock`].

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::waitqueue::WakerRegistration;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_framebuf::{FrameBuf, backends::DMACapableFrameBufferBackend};

use crate::display_backend::{AsyncDmaTransfer, DmaTransfer};

/// DMA transfer token using Embassy's [`WakerRegistration`] wake path.
pub struct EmbassyWaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    framebuffer: Option<FrameBuf<Rgb565, FB>>,
    done: AtomicBool,
    waker: WakerRegistration,
}

impl<FB> EmbassyWaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    pub fn new(framebuffer: FrameBuf<Rgb565, FB>) -> Self {
        Self {
            framebuffer: Some(framebuffer),
            done: AtomicBool::new(false),
            waker: WakerRegistration::new(),
        }
    }

    /// Wake a task waiting on this transfer; call from the DMA ISR.
    pub fn signal_complete(&mut self) {
        self.done.store(true, Ordering::Release);
        self.waker.wake();
    }
}

impl<FB> DmaTransfer for EmbassyWaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Buffer = FrameBuf<Rgb565, FB>;

    fn is_done(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    fn wait(self) -> Self::Buffer {
        while !self.is_done() {
            core::hint::spin_loop();
        }
        self.framebuffer
            .expect("EmbassyWaitTransfer consumed after completion")
    }
}

/// Future adapter for [`EmbassyWaitTransfer`].
pub struct EmbassyWaitTransferFuture<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    xfer: Option<EmbassyWaitTransfer<FB>>,
}

impl<FB> AsyncDmaTransfer for EmbassyWaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type WaitFuture = EmbassyWaitTransferFuture<FB>;

    fn wait_async(self) -> Self::WaitFuture {
        EmbassyWaitTransferFuture { xfer: Some(self) }
    }
}

impl<FB> core::future::Future for EmbassyWaitTransferFuture<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Output = FrameBuf<Rgb565, FB>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let xfer = this
            .xfer
            .as_mut()
            .expect("EmbassyWaitTransferFuture polled after completion");
        if xfer.is_done() {
            return core::task::Poll::Ready(
                this.xfer
                    .take()
                    .expect("EmbassyWaitTransferFuture polled after completion")
                    .framebuffer
                    .expect("EmbassyWaitTransferFuture polled after completion"),
            );
        }
        xfer.waker.register(cx.waker());
        if xfer.is_done() {
            core::task::Poll::Ready(
                this.xfer
                    .take()
                    .expect("EmbassyWaitTransferFuture polled after completion")
                    .framebuffer
                    .expect("EmbassyWaitTransferFuture polled after completion"),
            )
        } else {
            core::task::Poll::Pending
        }
    }
}

/// Monotonic frame delta helper.
///
/// Uses [`embassy_time::Instant`] on `no_std` firmware and `std::time::Instant` when the
/// crate's `std` feature is enabled (host examples/tests).
pub struct FrameClock {
    #[cfg(feature = "std")]
    last: Option<std::time::Instant>,
    #[cfg(not(feature = "std"))]
    last: Option<embassy_time::Instant>,
}

impl FrameClock {
    pub const fn new() -> Self {
        Self { last: None }
    }

    /// Milliseconds since the previous call (capped at `u32::MAX`).
    pub fn tick_ms(&mut self) -> u32 {
        #[cfg(feature = "std")]
        {
            let now = std::time::Instant::now();
            let dt = match self.last {
                Some(prev) => now.duration_since(prev).as_millis(),
                None => 0,
            };
            self.last = Some(now);
            dt.min(u32::MAX as u128) as u32
        }
        #[cfg(not(feature = "std"))]
        {
            let now = embassy_time::Instant::now();
            let dt = match self.last {
                Some(prev) => (now - prev).as_millis(),
                None => 0,
            };
            self.last = Some(now);
            dt.min(u32::MAX as u64) as u32
        }
    }
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::new()
    }
}
