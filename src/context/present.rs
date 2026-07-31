//! Present helpers for swap-chain integration.

use embedded_graphics_framebuf::backends::DMACapableFrameBufferBackend;

use crate::{
    GuiContext,
    display_backend::{AsyncDmaTransfer, DisplayBackend, DisplayError},
    swapchain::SwapChain,
};

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    /// Non-blocking present of the dirty bounding box after rendering into the back buffer.
    ///
    /// Returns [`DisplayError::Busy`] if a DMA transfer is still in flight. For multiple
    /// disjoint dirty rects, this issues one partial DMA covering the bounding box.
    pub fn try_present_dirty<const W: usize, const H: usize, FB, B>(
        &self,
        swap: &mut SwapChain<W, H, FB, B>,
    ) -> Result<(), DisplayError>
    where
        FB: DMACapableFrameBufferBackend<Color = embedded_graphics_core::pixelcolor::Rgb565>,
        B: DisplayBackend<W, H, FB>,
    {
        let Some(region) = self.bounding_present_region() else {
            return Ok(());
        };
        if region.is_empty() {
            return Ok(());
        }
        swap.try_present_region(region)
    }

    /// Async present of the dirty bounding box after rendering into the back buffer.
    pub async fn present_dirty_async<const W: usize, const H: usize, FB, B>(
        &self,
        swap: &mut SwapChain<W, H, FB, B>,
    ) -> Result<(), DisplayError>
    where
        FB: DMACapableFrameBufferBackend<Color = embedded_graphics_core::pixelcolor::Rgb565>,
        B: DisplayBackend<W, H, FB>,
        B::Transfer: AsyncDmaTransfer<
            Buffer = embedded_graphics_framebuf::FrameBuf<
                embedded_graphics_core::pixelcolor::Rgb565,
                FB,
            >,
        >,
    {
        let Some(region) = self.bounding_present_region() else {
            return Ok(());
        };
        if region.is_empty() {
            return Ok(());
        }
        swap.present_region_async(region).await
    }

    /// Full-frame non-blocking present (ignores dirty region geometry).
    pub fn try_present_frame<const W: usize, const H: usize, FB, B>(
        &self,
        swap: &mut SwapChain<W, H, FB, B>,
    ) -> Result<(), DisplayError>
    where
        FB: DMACapableFrameBufferBackend<Color = embedded_graphics_core::pixelcolor::Rgb565>,
        B: DisplayBackend<W, H, FB>,
    {
        let _ = self;
        swap.try_present()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::{
        display_backend::SimulatorBackend, geometry::Rect, style::Style,
        swapchain::StandardSwapChain,
    };
    use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
    use std::vec;

    fn make_swap() -> StandardSwapChain<64, 64, SimulatorBackend> {
        let fb0: &'static mut [Rgb565] = vec![Rgb565::BLACK; 64 * 64].leak();
        let fb1: &'static mut [Rgb565] = vec![Rgb565::BLACK; 64 * 64].leak();
        StandardSwapChain::from_static_slices(fb0, fb1, false, SimulatorBackend::new())
    }

    #[test]
    fn try_present_dirty_uses_bounding_region() {
        let mut gui = GuiContext::<4, 4, 4>::new(Rect::new(0, 0, 64, 64));
        gui.add_label(Rect::new(4, 4, 20, 8), "hi", Style::label())
            .unwrap();
        let mut swap = make_swap();
        assert!(gui.try_present_dirty(&mut swap).is_ok());
    }
}
