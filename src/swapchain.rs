//! Swap chain implementation for double-buffered rendering.
//!
//! A swap chain manages two framebuffers (front and back) and coordinates
//! DMA transfers to eliminate visual tearing and improve performance.
//!
//! # Architecture
//! - **Back buffer**: The CPU renders into this buffer at all times.
//! - **Front buffer**: Owned by either an in-flight DMA transfer or the
//!   swap chain itself while idle.
//! - **Swap**: When rendering completes, `present()` recovers the front
//!   buffer (waiting for any in-flight DMA), swaps it with the back
//!   buffer, and hands the new front to the backend to start the next
//!   transfer.
//!
//! # Safety
//! The front framebuffer is moved into the [`DmaTransfer`] token returned
//! by the backend. The compiler enforces that nobody can write to it until
//! [`DmaTransfer::wait`] returns it, eliminating the data race that a
//! borrow-based API cannot prevent.

use crate::display_backend::{DisplayBackend, DisplayError, DisplayRegion, DmaTransfer};
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_framebuf::{
    FrameBuf,
    backends::{DMACapableFrameBufferBackend, EndianCorrectedBuffer, EndianCorrection},
};

// ── FrontState ────────────────────────────────────────────────────────────────

/// Tracks whether the front framebuffer is idle (owned by the swap chain)
/// or in-flight (owned by a DMA transfer token).
enum FrontState<FB, Xfer> {
    Idle(FB),
    InFlight(Xfer),
}

impl<FB, Xfer: DmaTransfer<Buffer = FB>> FrontState<FB, Xfer> {
    /// Recover the framebuffer, blocking if a transfer is still running.
    fn recover(self) -> FB {
        match self {
            FrontState::Idle(fb) => fb,
            FrontState::InFlight(xfer) => xfer.wait(),
        }
    }

    /// Returns `true` if there is no in-flight transfer or it has finished.
    fn is_ready(&self) -> bool {
        match self {
            FrontState::Idle(_) => true,
            FrontState::InFlight(xfer) => xfer.is_done(),
        }
    }
}

impl<FB, Xfer: crate::display_backend::AsyncDmaTransfer<Buffer = FB>> FrontState<FB, Xfer> {
    /// Recover the framebuffer asynchronously.
    async fn recover_async(self) -> FB {
        match self {
            FrontState::Idle(fb) => fb,
            FrontState::InFlight(xfer) => xfer.wait_async().await,
        }
    }
}

// ── SwapChain ─────────────────────────────────────────────────────────────────

/// Double-buffered swap chain for tear-free rendering.
///
/// The back buffer is always available to the CPU via [`get_back_buffer`].
/// The front buffer moves into the backend's DMA transfer token on each
/// [`present`] call and is recovered (blocking) on the next one.
///
/// # Type Parameters
/// - `W`, `H`: Framebuffer dimensions in pixels (const generics).
/// - `FB`: Framebuffer backend implementing [`DMACapableFrameBufferBackend`].
/// - `B`: Display backend implementing [`DisplayBackend<W, H, FB>`].
///
/// [`get_back_buffer`]: SwapChain::get_back_buffer
/// [`present`]: SwapChain::present
pub struct SwapChain<const W: usize, const H: usize, FB, B>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
    B: DisplayBackend<W, H, FB>,
{
    /// Back buffer — always owned by the swap chain.
    back: FrameBuf<Rgb565, FB>,
    /// Front buffer — either idle here or inside a DMA transfer token.
    front: Option<FrontState<FrameBuf<Rgb565, FB>, B::Transfer>>,
    backend: B,
    frame_count: u64,
}

/// Type alias for `SwapChain` backed by [`EndianCorrectedBuffer`].
///
/// This is the most common configuration for statically allocated
/// framebuffer memory.
pub type StandardSwapChain<const W: usize, const H: usize, B> =
    SwapChain<W, H, EndianCorrectedBuffer<'static, Rgb565>, B>;

// ── StandardSwapChain constructor ─────────────────────────────────────────────

impl<const W: usize, const H: usize, B> StandardSwapChain<W, H, B>
where
    B: DisplayBackend<W, H, EndianCorrectedBuffer<'static, Rgb565>>,
{
    /// Create a new swap chain from static slices.
    ///
    /// # Arguments
    /// * `front_data` — Static mutable slice for the front framebuffer.
    /// * `back_data`  — Static mutable slice for the back framebuffer.
    /// * `big_endian` — Byte order of pixel data sent to the display.
    /// * `backend`    — Display backend used for DMA operations.
    ///
    /// # Example
    /// ```ignore
    /// static mut FB0: [Rgb565; 240 * 135] = [Rgb565::BLACK; 240 * 135];
    /// static mut FB1: [Rgb565; 240 * 135] = [Rgb565::BLACK; 240 * 135];
    ///
    /// let swap_chain = unsafe {
    ///     StandardSwapChain::<240, 135, _>::from_static_slices(
    ///         &mut FB0,
    ///         &mut FB1,
    ///         false,
    ///         MyHardwareBackend::new(),
    ///     )
    /// };
    /// ```
    pub fn from_static_slices(
        front_data: &'static mut [Rgb565],
        back_data: &'static mut [Rgb565],
        big_endian: bool,
        backend: B,
    ) -> Self {
        let mk_buf = |data: &'static mut [Rgb565]| {
            let correction = if big_endian {
                EndianCorrection::ToBigEndian
            } else {
                EndianCorrection::ToLittleEndian
            };
            EndianCorrectedBuffer::new(data, correction)
        };

        let front_fb = FrameBuf::new(mk_buf(front_data), W, H);
        let back_fb = FrameBuf::new(mk_buf(back_data), W, H);

        Self {
            back: back_fb,
            front: Some(FrontState::Idle(front_fb)),
            backend,
            frame_count: 0,
        }
    }
}

// ── SwapChain methods ─────────────────────────────────────────────────────────

impl<const W: usize, const H: usize, FB, B> SwapChain<W, H, FB, B>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
    B: DisplayBackend<W, H, FB>,
{
    /// Get a mutable reference to the back buffer for rendering.
    ///
    /// The back buffer is always available — DMA only ever touches the
    /// front buffer, which is kept separately.
    pub fn get_back_buffer(&mut self) -> &mut FrameBuf<Rgb565, FB> {
        &mut self.back
    }

    /// Get a reference to the front buffer if it is currently idle.
    ///
    /// Returns `None` while a DMA transfer is in progress.
    pub fn get_front_buffer(&self) -> Option<&FrameBuf<Rgb565, FB>> {
        match &self.front {
            Some(FrontState::Idle(fb)) => Some(fb),
            _ => None,
        }
    }

    /// Present the back buffer (blocking).
    ///
    /// 1. Waits for any in-progress DMA transfer to complete.
    /// 2. Swaps the front and back buffers.
    /// 3. Starts a new DMA transfer of the new front buffer.
    ///
    /// After this call returns, the CPU may immediately start rendering to
    /// the new back buffer while DMA reads from the new front buffer.
    pub fn present(&mut self) -> Result<(), DisplayError> {
        self.present_impl(|backend, fb| backend.start_dma_transfer(fb))
    }

    /// Present the back buffer without blocking.
    ///
    /// Returns [`DisplayError::Busy`] immediately if a DMA transfer is
    /// still in progress, leaving both buffers unchanged.
    pub fn try_present(&mut self) -> Result<(), DisplayError> {
        if !self.is_ready() {
            return Err(DisplayError::Busy);
        }
        self.present_impl(|backend, fb| backend.start_dma_transfer(fb))
    }

    /// Present only a sub-region of the back buffer (blocking).
    ///
    /// Backends that do not support partial DMA automatically fall back to
    /// a full-frame transfer.
    pub fn present_region(&mut self, region: DisplayRegion) -> Result<(), DisplayError> {
        self.present_impl(|backend, fb| backend.start_dma_transfer_region(fb, region))
    }

    /// Non-blocking partial present.
    ///
    /// Returns [`DisplayError::Busy`] if a transfer is still running.
    pub fn try_present_region(&mut self, region: DisplayRegion) -> Result<(), DisplayError> {
        if !self.is_ready() {
            return Err(DisplayError::Busy);
        }
        self.present_impl(|backend, fb| backend.start_dma_transfer_region(fb, region))
    }

    /// Block until the current DMA transfer completes.
    ///
    /// After this call the front buffer is in the idle state and the next
    /// `present` will not need to wait.
    pub fn wait_for_vsync(&mut self) {
        if let Some(state) = self.front.take() {
            let fb = state.recover();
            self.front = Some(FrontState::Idle(fb));
        }
    }

    /// Wait for the current DMA transfer to complete asynchronously.
    pub async fn wait_for_vsync_async(&mut self)
    where
        B::Transfer: crate::display_backend::AsyncDmaTransfer<Buffer = FrameBuf<Rgb565, FB>>,
    {
        if let Some(state) = self.front.take() {
            let fb = state.recover_async().await;
            self.front = Some(FrontState::Idle(fb));
        }
    }

    /// Present the back buffer asynchronously.
    pub async fn present_async(&mut self) -> Result<(), DisplayError>
    where
        B::Transfer: crate::display_backend::AsyncDmaTransfer<Buffer = FrameBuf<Rgb565, FB>>,
    {
        // 1. Recover the front framebuffer asynchronously (yield if DMA was running).
        let old_front = if let Some(state) = self.front.take() {
            state.recover_async().await
        } else {
            panic!("SwapChain front buffer missing — double present?");
        };

        // 2. Swap: old_front becomes the new back, current back becomes the new front.
        let new_front = core::mem::replace(&mut self.back, old_front);

        // 3. Hand the new front to the backend.
        match self.backend.start_dma_transfer(new_front) {
            Ok(transfer) => {
                self.front = Some(FrontState::InFlight(transfer));
                self.frame_count += 1;
                Ok(())
            }
            Err(e) => {
                let recovered_front = core::mem::replace(&mut self.back, e.framebuffer);
                self.front = Some(FrontState::Idle(recovered_front));
                Err(e.error)
            }
        }
    }

    /// Present only a sub-region of the back buffer asynchronously.
    pub async fn present_region_async(&mut self, region: DisplayRegion) -> Result<(), DisplayError>
    where
        B::Transfer: crate::display_backend::AsyncDmaTransfer<Buffer = FrameBuf<Rgb565, FB>>,
    {
        // 1. Recover the front framebuffer asynchronously (yield if DMA was running).
        let old_front = if let Some(state) = self.front.take() {
            state.recover_async().await
        } else {
            panic!("SwapChain front buffer missing — double present?");
        };

        // 2. Swap: old_front becomes the new back, current back becomes the new front.
        let new_front = core::mem::replace(&mut self.back, old_front);

        // 3. Hand the new front to the backend.
        match self.backend.start_dma_transfer_region(new_front, region) {
            Ok(transfer) => {
                self.front = Some(FrontState::InFlight(transfer));
                self.frame_count += 1;
                Ok(())
            }
            Err(e) => {
                let recovered_front = core::mem::replace(&mut self.back, e.framebuffer);
                self.front = Some(FrontState::Idle(recovered_front));
                Err(e.error)
            }
        }
    }

    /// Returns `true` if no DMA transfer is running (or the hardware has
    /// signalled completion), so `try_present` would succeed.
    pub fn is_ready(&self) -> bool {
        self.front.as_ref().is_none_or(|s| s.is_ready())
    }

    /// Total number of frames presented since construction (or the last
    /// [`reset_frame_count`](SwapChain::reset_frame_count) call).
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Reset the frame counter to zero.
    pub fn reset_frame_count(&mut self) {
        self.frame_count = 0;
    }

    /// Framebuffer dimensions `(W, H)`.
    pub fn dimensions(&self) -> (usize, usize) {
        (W, H)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Shared logic for all present variants.
    ///
    /// `start_fn` is called with `(&mut backend, front_framebuffer)` and
    /// must return a transfer token or a `TransferError`.
    fn present_impl<F>(&mut self, start_fn: F) -> Result<(), DisplayError>
    where
        F: FnOnce(
            &mut B,
            FrameBuf<Rgb565, FB>,
        ) -> Result<B::Transfer, crate::display_backend::TransferError<FB>>,
    {
        // 1. Recover the front framebuffer (block if DMA was running).
        let old_front = self
            .front
            .take()
            .expect("SwapChain front buffer missing — double present?")
            .recover();

        // 2. Swap: old_front becomes the new back, current back becomes the
        //    new front.
        let new_front = core::mem::replace(&mut self.back, old_front);

        // 3. Hand the new front to the backend.
        match start_fn(&mut self.backend, new_front) {
            Ok(transfer) => {
                self.front = Some(FrontState::InFlight(transfer));
                self.frame_count += 1;
                Ok(())
            }
            Err(e) => {
                // Transfer failed — put the buffer back so it is not lost.
                // self.back currently holds old_front; swap back.
                let recovered_front = core::mem::replace(&mut self.back, e.framebuffer);
                self.front = Some(FrontState::Idle(recovered_front));
                Err(e.error)
            }
        }
    }
}

// ── TripleSwapChain ───────────────────────────────────────────────────────────

/// Triple-buffered swap chain for smoother pacing under bursty frame times.
///
/// - `render`: the buffer the CPU is currently writing to.
/// - `ready`:  the last fully-rendered buffer waiting to be shown.
/// - `display`: the buffer currently owned by DMA (or idle between frames).
///
/// On each `present` call the render and display buffers are rotated so
/// the CPU can immediately start the next frame without waiting for the
/// display scan-out to finish.
#[cfg(feature = "triple-buffering")]
pub struct TripleSwapChain<const W: usize, const H: usize, FB, B>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
    B: DisplayBackend<W, H, FB>,
{
    display: Option<FrontState<FrameBuf<Rgb565, FB>, B::Transfer>>,
    ready: FrameBuf<Rgb565, FB>,
    render: FrameBuf<Rgb565, FB>,
    backend: B,
    frame_count: u64,
}

#[cfg(feature = "triple-buffering")]
pub type StandardTripleSwapChain<const W: usize, const H: usize, B> =
    TripleSwapChain<W, H, EndianCorrectedBuffer<'static, Rgb565>, B>;

#[cfg(feature = "triple-buffering")]
impl<const W: usize, const H: usize, B> StandardTripleSwapChain<W, H, B>
where
    B: DisplayBackend<W, H, EndianCorrectedBuffer<'static, Rgb565>>,
{
    pub fn from_static_slices(
        display_data: &'static mut [Rgb565],
        ready_data: &'static mut [Rgb565],
        render_data: &'static mut [Rgb565],
        big_endian: bool,
        backend: B,
    ) -> Self {
        let mk = |data: &'static mut [Rgb565]| {
            let correction = if big_endian {
                EndianCorrection::ToBigEndian
            } else {
                EndianCorrection::ToLittleEndian
            };
            FrameBuf::new(EndianCorrectedBuffer::new(data, correction), W, H)
        };
        Self {
            display: Some(FrontState::Idle(mk(display_data))),
            ready: mk(ready_data),
            render: mk(render_data),
            backend,
            frame_count: 0,
        }
    }
}

#[cfg(feature = "triple-buffering")]
impl<const W: usize, const H: usize, FB, B> TripleSwapChain<W, H, FB, B>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
    B: DisplayBackend<W, H, FB>,
{
    /// Get a mutable reference to the render buffer.
    pub fn get_render_buffer(&mut self) -> &mut FrameBuf<Rgb565, FB> {
        &mut self.render
    }

    /// Present the render buffer (blocking).
    ///
    /// Waits for the previous display transfer to complete, then:
    /// 1. Rotates `render → display` and starts DMA.
    /// 2. Rotates `display (old) → ready` so the CPU has a fresh buffer.
    pub fn present(&mut self) -> Result<(), DisplayError> {
        self.present_impl(|backend, fb| backend.start_dma_transfer(fb))
    }

    /// Present the render buffer asynchronously.
    pub async fn present_async(&mut self) -> Result<(), DisplayError>
    where
        B::Transfer: crate::display_backend::AsyncDmaTransfer<Buffer = FrameBuf<Rgb565, FB>>,
    {
        // 1. Recover the display buffer asynchronously (yield if DMA was running).
        let old_display = if let Some(state) = self.display.take() {
            state.recover_async().await
        } else {
            panic!("TripleSwapChain display buffer missing");
        };

        // 2. render → new display, old_display → render slot temporarily.
        let rendered = core::mem::replace(&mut self.render, old_display);

        // 3. Start DMA on the freshly rendered frame.
        match self.backend.start_dma_transfer(rendered) {
            Ok(transfer) => {
                self.display = Some(FrontState::InFlight(transfer));
                // 4. ready ↔ render: CPU gets the old ready buffer to render into.
                core::mem::swap(&mut self.ready, &mut self.render);
                self.frame_count += 1;
                Ok(())
            }
            Err(e) => {
                let old_display = core::mem::replace(&mut self.render, e.framebuffer);
                self.display = Some(FrontState::Idle(old_display));
                Err(e.error)
            }
        }
    }

    /// Non-blocking triple-buffer present.
    ///
    /// Returns [`DisplayError::Busy`] if the previous display transfer has
    /// not finished yet.
    pub fn try_present(&mut self) -> Result<(), DisplayError> {
        let ready = self.display.as_ref().is_none_or(|s| s.is_ready());
        if !ready {
            return Err(DisplayError::Busy);
        }
        self.present_impl(|backend, fb| backend.start_dma_transfer(fb))
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    fn present_impl<F>(&mut self, start_fn: F) -> Result<(), DisplayError>
    where
        F: FnOnce(
            &mut B,
            FrameBuf<Rgb565, FB>,
        ) -> Result<B::Transfer, crate::display_backend::TransferError<FB>>,
    {
        // 1. Recover the display buffer (block if DMA was running).
        let old_display = self
            .display
            .take()
            .expect("TripleSwapChain display buffer missing")
            .recover();

        // 2. render → new display, old_display → render slot temporarily.
        let rendered = core::mem::replace(&mut self.render, old_display);

        // 3. Start DMA on the freshly rendered frame.
        match start_fn(&mut self.backend, rendered) {
            Ok(transfer) => {
                self.display = Some(FrontState::InFlight(transfer));
                // 4. ready ↔ render: CPU gets the old ready buffer to render into.
                core::mem::swap(&mut self.ready, &mut self.render);
                self.frame_count += 1;
                Ok(())
            }
            Err(e) => {
                // Undo: put rendered back into render, restore old_display.
                let old_display = core::mem::replace(&mut self.render, e.framebuffer);
                self.display = Some(FrontState::Idle(old_display));
                Err(e.error)
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::display_backend::{DmaTransfer, SimulatorBackend, TransferError};
    use core::cell::Cell;
    use embedded_graphics_core::pixelcolor::RgbColor;
    use std::vec;

    fn make_static_slice(n: usize) -> &'static mut [Rgb565] {
        vec![Rgb565::BLACK; n].leak()
    }

    // ── TrackingBackend ───────────────────────────────────────────────────────
    //
    // Counts how many times start_dma_transfer_region was called and
    // otherwise behaves like SimulatorBackend.

    struct TrackingTransfer<FB: DMACapableFrameBufferBackend<Color = Rgb565>> {
        framebuffer: Option<FrameBuf<Rgb565, FB>>,
    }

    impl<FB: DMACapableFrameBufferBackend<Color = Rgb565>> DmaTransfer for TrackingTransfer<FB> {
        type Buffer = FrameBuf<Rgb565, FB>;
        fn is_done(&self) -> bool {
            true
        }
        fn wait(mut self) -> FrameBuf<Rgb565, FB> {
            self.framebuffer.take().unwrap()
        }
    }

    impl<FB: DMACapableFrameBufferBackend<Color = Rgb565>> core::future::Future
        for TrackingTransfer<FB>
    {
        type Output = FrameBuf<Rgb565, FB>;
        fn poll(
            self: core::pin::Pin<&mut Self>,
            _cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<Self::Output> {
            core::task::Poll::Ready(
                unsafe { self.get_unchecked_mut() }
                    .framebuffer
                    .take()
                    .unwrap(),
            )
        }
    }

    impl<FB: DMACapableFrameBufferBackend<Color = Rgb565>> crate::display_backend::AsyncDmaTransfer
        for TrackingTransfer<FB>
    {
        type WaitFuture = Self;
        fn wait_async(self) -> Self::WaitFuture {
            self
        }
    }

    struct TrackingBackend {
        region_present_count: Cell<usize>,
    }

    impl TrackingBackend {
        fn new() -> Self {
            Self {
                region_present_count: Cell::new(0),
            }
        }
    }

    impl<const W: usize, const H: usize, FB> DisplayBackend<W, H, FB> for TrackingBackend
    where
        FB: DMACapableFrameBufferBackend<Color = Rgb565>,
    {
        type Transfer = TrackingTransfer<FB>;

        fn start_dma_transfer(
            &mut self,
            framebuffer: FrameBuf<Rgb565, FB>,
        ) -> Result<TrackingTransfer<FB>, TransferError<FB>> {
            Ok(TrackingTransfer {
                framebuffer: Some(framebuffer),
            })
        }

        fn start_dma_transfer_region(
            &mut self,
            framebuffer: FrameBuf<Rgb565, FB>,
            _region: DisplayRegion,
        ) -> Result<TrackingTransfer<FB>, TransferError<FB>> {
            self.region_present_count
                .set(self.region_present_count.get() + 1);
            Ok(TrackingTransfer {
                framebuffer: Some(framebuffer),
            })
        }
    }

    // ── Helper ────────────────────────────────────────────────────────────────

    fn make_swap_chain<B>(backend: B) -> StandardSwapChain<320, 240, B>
    where
        B: DisplayBackend<320, 240, EndianCorrectedBuffer<'static, Rgb565>>,
    {
        StandardSwapChain::<320, 240, _>::from_static_slices(
            make_static_slice(320 * 240),
            make_static_slice(320 * 240),
            false,
            backend,
        )
    }

    // ── SwapChain tests ───────────────────────────────────────────────────────

    #[test]
    fn test_swapchain_creation() {
        let sc = make_swap_chain(SimulatorBackend::new());
        assert_eq!(sc.dimensions(), (320, 240));
        assert_eq!(sc.frame_count(), 0);
        assert!(sc.is_ready());
    }

    #[test]
    fn test_swapchain_present() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        assert!(sc.present().is_ok());
        assert_eq!(sc.frame_count(), 1);
    }

    #[test]
    fn test_swapchain_multiple_presents() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        for _ in 0..5 {
            assert!(sc.present().is_ok());
        }
        assert_eq!(sc.frame_count(), 5);
    }

    #[test]
    fn test_swapchain_try_present() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        assert!(sc.try_present().is_ok());
        assert_eq!(sc.frame_count(), 1);
    }

    #[test]
    fn test_swapchain_frame_counter() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        assert_eq!(sc.frame_count(), 0);
        sc.present().unwrap();
        assert_eq!(sc.frame_count(), 1);
        sc.present().unwrap();
        assert_eq!(sc.frame_count(), 2);
        sc.reset_frame_count();
        assert_eq!(sc.frame_count(), 0);
    }

    #[test]
    fn test_swapchain_get_back_buffer_always_available() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        sc.present().unwrap();
        // Even after present, back buffer must be accessible for rendering
        let _back = sc.get_back_buffer();
    }

    #[test]
    fn test_swapchain_wait_for_vsync() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        sc.present().unwrap();
        sc.wait_for_vsync();
        // After vsync, front is idle and is_ready returns true
        assert!(sc.is_ready());
    }

    #[test]
    fn test_swapchain_present_region() {
        let fb0 = make_static_slice(64 * 64);
        let fb1 = make_static_slice(64 * 64);
        let mut sc = StandardSwapChain::<64, 64, _>::from_static_slices(
            fb0,
            fb1,
            false,
            TrackingBackend::new(),
        );
        sc.present_region(DisplayRegion::new(0, 0, 8, 8)).unwrap();
        assert_eq!(sc.backend.region_present_count.get(), 1);
    }

    #[test]
    fn test_swapchain_is_ready_after_simulator_present() {
        let mut sc = make_swap_chain(SimulatorBackend::new());
        // SimulatorBackend transfer is always done immediately
        sc.present().unwrap();
        assert!(sc.is_ready());
    }

    // ── TripleSwapChain tests ─────────────────────────────────────────────────

    #[cfg(feature = "triple-buffering")]
    #[test]
    fn test_triple_swapchain_present() {
        let fb0 = make_static_slice(64 * 64);
        let fb1 = make_static_slice(64 * 64);
        let fb2 = make_static_slice(64 * 64);
        let mut sc = StandardTripleSwapChain::<64, 64, _>::from_static_slices(
            fb0,
            fb1,
            fb2,
            false,
            SimulatorBackend::new(),
        );
        assert_eq!(sc.frame_count(), 0);
        sc.present().unwrap();
        assert_eq!(sc.frame_count(), 1);
    }
}
