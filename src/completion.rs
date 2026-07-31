//! ISR-safe DMA completion signaling for async and sync present paths.
//!
//! [`CompletionSlot`] is runtime-agnostic: call [`CompletionSlot::signal`] from a
//! DMA ISR, poll with [`CompletionSlot::is_signaled`] in RTIC/bare-metal tasks, or
//! await via [`WaitTransfer`] / [`WaitTransferFuture`] under Embassy.

use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};

use critical_section::Mutex;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_framebuf::{FrameBuf, backends::DMACapableFrameBufferBackend};

use crate::display_backend::{AsyncDmaTransfer, DmaTransfer};

/// One-shot completion flag with optional async waker notification.
///
/// Reset before starting DMA, signal from the transfer-complete ISR.
pub struct CompletionSlot {
    signaled: AtomicBool,
    waker: Mutex<Cell<Option<Waker>>>,
}

impl CompletionSlot {
    /// Create a cleared completion slot.
    pub const fn new() -> Self {
        Self {
            signaled: AtomicBool::new(false),
            waker: Mutex::new(Cell::new(None)),
        }
    }

    /// Clear the slot before kicking off a new DMA transfer.
    pub fn reset(&self) {
        self.signaled.store(false, Ordering::Release);
        critical_section::with(|cs| {
            self.waker.borrow(cs).set(None);
        });
    }

    /// Mark complete and wake any registered async task.
    ///
    /// Safe to call from interrupt context.
    pub fn signal(&self) {
        self.signaled.store(true, Ordering::Release);
        critical_section::with(|cs| {
            if let Some(waker) = self.waker.borrow(cs).take() {
                waker.wake();
            }
        });
    }
}

impl Default for CompletionSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionSlot {
    /// Returns `true` after [`signal`](Self::signal) until the next [`reset`](Self::reset).
    pub fn is_signaled(&self) -> bool {
        self.signaled.load(Ordering::Acquire)
    }

    /// Poll for completion, registering `cx`'s waker when still pending.
    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<()> {
        if self.is_signaled() {
            return Poll::Ready(());
        }
        critical_section::with(|cs| {
            self.waker.borrow(cs).set(Some(cx.waker().clone()));
        });
        if self.is_signaled() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Reference [`AsyncDmaTransfer`] token backed by a [`CompletionSlot`].
pub struct WaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    framebuffer: Option<FrameBuf<Rgb565, FB>>,
    completion: &'static CompletionSlot,
}

impl<FB> WaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    /// Build a transfer token; `completion` must outlive all in-flight DMA ops.
    pub fn new(framebuffer: FrameBuf<Rgb565, FB>, completion: &'static CompletionSlot) -> Self {
        completion.reset();
        Self {
            framebuffer: Some(framebuffer),
            completion,
        }
    }

    /// The completion slot wired to this transfer (for ISR handlers).
    pub const fn completion(&self) -> &'static CompletionSlot {
        self.completion
    }
}

impl<FB> DmaTransfer for WaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Buffer = FrameBuf<Rgb565, FB>;

    fn is_done(&self) -> bool {
        self.completion.is_signaled()
    }

    fn wait(self) -> Self::Buffer {
        while !self.completion.is_signaled() {
            core::hint::spin_loop();
        }
        self.framebuffer
            .expect("WaitTransfer polled after completion")
    }
}

/// Future returned by [`WaitTransfer::wait_async`].
pub struct WaitTransferFuture<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    inner: Option<WaitTransfer<FB>>,
}

impl<FB> AsyncDmaTransfer for WaitTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type WaitFuture = WaitTransferFuture<FB>;

    fn wait_async(self) -> Self::WaitFuture {
        WaitTransferFuture { inner: Some(self) }
    }
}

impl<FB> Future for WaitTransferFuture<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Output = FrameBuf<Rgb565, FB>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let inner = this
            .inner
            .as_mut()
            .expect("WaitTransferFuture polled after completion");
        match inner.completion.poll_wait(cx) {
            Poll::Ready(()) => Poll::Ready(
                this.inner
                    .take()
                    .expect("WaitTransferFuture polled after completion")
                    .framebuffer
                    .expect("WaitTransferFuture polled after completion"),
            ),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_graphics_core::pixelcolor::RgbColor;
    use embedded_graphics_framebuf::backends::{EndianCorrectedBuffer, EndianCorrection};
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    type TestBackend = EndianCorrectedBuffer<'static, Rgb565>;

    struct TestWake(Arc<AtomicBool>);

    impl Wake for TestWake {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::Release);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.store(true, Ordering::Release);
        }
    }

    fn make_fb() -> FrameBuf<Rgb565, TestBackend> {
        let data: &'static mut [Rgb565] = std::vec![Rgb565::BLACK; 4].leak();
        FrameBuf::new(
            EndianCorrectedBuffer::new(data, EndianCorrection::ToLittleEndian),
            2,
            2,
        )
    }

    #[test]
    fn completion_slot_sync_wait() {
        static DONE: CompletionSlot = CompletionSlot::new();
        let fb = make_fb();
        let xfer = WaitTransfer::new(fb, &DONE);
        assert!(!xfer.is_done());
        DONE.signal();
        assert!(xfer.is_done());
        let _ = xfer.wait();
    }

    #[test]
    fn completion_slot_async_wake() {
        static DONE: CompletionSlot = CompletionSlot::new();
        let fb = make_fb();
        let xfer = WaitTransfer::new(fb, &DONE);
        let mut fut = xfer.wait_async();
        let woken = Arc::new(AtomicBool::new(false));
        let waker = Waker::from(Arc::new(TestWake(Arc::clone(&woken))));
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(Pin::new(&mut fut).poll(&mut cx), Poll::Pending));
        DONE.signal();
        assert!(matches!(Pin::new(&mut fut).poll(&mut cx), Poll::Ready(_)));
    }
}
