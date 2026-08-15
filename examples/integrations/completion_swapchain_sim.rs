//! Demonstrates runtime-agnostic async present with [`CompletionSlot`].
//!
//! Run: `cargo run --example completion_swapchain_sim --features std`

use core::future::Future;
use core::task::{Context, Poll, Waker};
use std::pin::pin;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::task::Wake;

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_framebuf::backends::EndianCorrectedBuffer;
use embedded_gui::{CompletionSlot, StandardSwapChain, WaitTransfer};

struct FlagWake(Arc<AtomicBool>);

impl Wake for FlagWake {
    fn wake(self: Arc<Self>) {
        self.0.store(true, Ordering::Release);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, Ordering::Release);
    }
}

fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let flag = Arc::new(AtomicBool::new(false));
    let waker = Waker::from(Arc::new(FlagWake(Arc::clone(&flag))));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                while !flag.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                flag.store(false, Ordering::Release);
            }
        }
    }
}

struct SlotBackend {
    slot: &'static CompletionSlot,
}

impl<const W: usize, const H: usize>
    embedded_gui::DisplayBackend<W, H, EndianCorrectedBuffer<'static, Rgb565>> for SlotBackend
{
    type Transfer = WaitTransfer<EndianCorrectedBuffer<'static, Rgb565>>;

    fn start_dma_transfer(
        &mut self,
        framebuffer: FrameBuf<Rgb565, EndianCorrectedBuffer<'static, Rgb565>>,
    ) -> Result<
        WaitTransfer<EndianCorrectedBuffer<'static, Rgb565>>,
        embedded_gui::TransferError<EndianCorrectedBuffer<'static, Rgb565>>,
    > {
        Ok(WaitTransfer::new(framebuffer, self.slot))
    }
}

fn main() {
    static DMA_DONE: CompletionSlot = CompletionSlot::new();

    let fb0: &'static mut [Rgb565] = vec![Rgb565::BLACK; 64 * 64].leak();
    let fb1: &'static mut [Rgb565] = vec![Rgb565::BLACK; 64 * 64].leak();
    let mut swap = StandardSwapChain::<64, 64, _>::from_static_slices(
        fb0,
        fb1,
        false,
        SlotBackend { slot: &DMA_DONE },
    );

    // Simulate rendering into the back buffer, then async present.
    let present = swap.present_async();
    // DMA completes "later" on another thread.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(5));
        DMA_DONE.signal();
    });
    block_on(present).expect("present failed");
    println!(
        "presented {} frame(s) via CompletionSlot + present_async",
        swap.frame_count()
    );
}
