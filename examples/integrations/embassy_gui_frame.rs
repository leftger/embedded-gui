//! Embassy [`FrameClock`] + async present using the core [`CompletionSlot`] path.
//!
//! Run: `cargo run --example embassy_gui_frame --features embassy,std`

use core::future::Future;
use core::task::{Context, Poll, Waker};
use std::pin::pin;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::task::Wake;

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::{
    FrameClock, GuiContext, StandardSwapChain, display_backend::SimulatorBackend, geometry::Rect,
    prelude::Style,
};

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

async fn ui_frame(
    gui: &mut GuiContext<'_, 8, 8, 8>,
    swap: &mut StandardSwapChain<128, 64, SimulatorBackend>,
    clock: &mut FrameClock,
) {
    let dt_ms = clock.tick_ms().max(16);
    let _ = gui.tick_input(dt_ms);
    // Render into `swap.get_back_buffer()` using your DrawTarget adapter, then present.
    let _ = swap.present_async().await;
    gui.clear_dirty();
}

fn main() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 128, 64));
    gui.add_label(Rect::new(4, 4, 80, 8), "Embassy path", Style::label())
        .unwrap();
    gui.clear_dirty();

    let fb0: &'static mut [Rgb565] = vec![Rgb565::BLACK; 128 * 64].leak();
    let fb1: &'static mut [Rgb565] = vec![Rgb565::BLACK; 128 * 64].leak();
    let mut swap = StandardSwapChain::from_static_slices(fb0, fb1, false, SimulatorBackend::new());
    let mut clock = FrameClock::new();

    block_on(ui_frame(&mut gui, &mut swap, &mut clock));
    println!(
        "Embassy FrameClock + present_async ok ({} frames)",
        swap.frame_count()
    );
}
