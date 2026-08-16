//! Display backend abstraction for DMA-based rendering.
//!
//! This module provides a platform-agnostic interface for asynchronous
//! framebuffer transfers using DMA (Direct Memory Access). This enables
//! double-buffered rendering where the CPU can render to one buffer while
//! the display hardware transfers another buffer.
//!
//! # Safety
//!
//! The key safety property of this API is that `start_dma_transfer` takes
//! ownership of the framebuffer and returns a [`DmaTransfer`] token. The
//! buffer is locked inside the token for the duration of the transfer —
//! the compiler prevents any access to it until [`DmaTransfer::wait`]
//! returns it. This eliminates the data race that arises from the
//! previous borrow-based API, where DMA could be reading from memory that
//! the CPU was free to overwrite.

use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_framebuf::{FrameBuf, backends::DMACapableFrameBufferBackend};

/// Rectangle region for partial framebuffer presents.
///
/// Alias for [`crate::PresentRegion`] so GUI dirty regions and DMA presents
/// share the same type.
pub use crate::present::PresentRegion as DisplayRegion;

/// Error types for display backend operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayError {
    /// DMA transfer is still in progress.
    Busy,
    /// Hardware error during transfer.
    HardwareError,
    /// Invalid buffer configuration.
    InvalidBuffer,
}

/// Returned when a DMA transfer fails to start.
///
/// Carries both the error code and the framebuffer back to the caller so
/// the buffer is not lost on failure.
pub struct TransferError<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    /// The framebuffer that could not be transferred.
    pub framebuffer: FrameBuf<Rgb565, FB>,
    /// The reason the transfer failed.
    pub error: DisplayError,
}

impl<FB> core::fmt::Debug for TransferError<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TransferError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

/// An in-progress DMA transfer that holds the framebuffer until completion.
///
/// The buffer is inaccessible while this token is live — the only way to
/// get it back is to call [`wait`](DmaTransfer::wait), which blocks until
/// the hardware has finished reading.
///
/// Implementors for real hardware should cancel the DMA in their [`Drop`]
/// impl so that dropping a token without waiting is always safe.
pub trait DmaTransfer {
    /// The buffer type that is returned when the transfer completes.
    type Buffer;

    /// Returns `true` if the DMA hardware has finished the transfer.
    fn is_done(&self) -> bool;

    /// Block until the transfer is complete and return the framebuffer.
    ///
    /// Consuming `self` ensures the buffer cannot be accessed while DMA
    /// is still reading it.
    fn wait(self) -> Self::Buffer;
}

/// An asynchronous DMA transfer that can be awaited as a future.
///
/// This allows integration with async executors (e.g. Embassy, RTOS task executors)
/// without blocking the CPU while the DMA transfer is in progress.
pub trait AsyncDmaTransfer: DmaTransfer {
    /// The future type returned by `wait_async`.
    type WaitFuture: core::future::Future<Output = Self::Buffer>;

    /// Return a future that resolves to the framebuffer once the DMA transfer completes.
    fn wait_async(self) -> Self::WaitFuture;
}

/// Platform-agnostic display backend trait.
///
/// Implementations handle the hardware-specific details of transferring a
/// framebuffer to the display. The API is intentionally ownership-based:
/// `start_dma_transfer` takes the framebuffer **by value** and returns a
/// [`DmaTransfer`] token. The buffer is held inside the token and cannot
/// be accessed again until [`DmaTransfer::wait`] returns it. This
/// prevents write-after-submit data races at compile time.
pub trait DisplayBackend<const W: usize, const H: usize, FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    /// The transfer token type returned by this backend.
    type Transfer: DmaTransfer<Buffer = FrameBuf<Rgb565, FB>>;

    /// Start a non-blocking DMA transfer of the full framebuffer.
    fn start_dma_transfer(
        &mut self,
        framebuffer: FrameBuf<Rgb565, FB>,
    ) -> Result<Self::Transfer, TransferError<FB>>;

    /// Start a non-blocking DMA transfer of a framebuffer sub-region.
    ///
    /// Backends that do not support partial transfers may ignore `region`
    /// and fall back to a full-frame transfer.
    fn start_dma_transfer_region(
        &mut self,
        framebuffer: FrameBuf<Rgb565, FB>,
        _region: DisplayRegion,
    ) -> Result<Self::Transfer, TransferError<FB>> {
        self.start_dma_transfer(framebuffer)
    }
}

// ── SimulatorBackend ──────────────────────────────────────────────────────────

/// A transfer token that is already complete.
pub struct CompletedTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    framebuffer: Option<FrameBuf<Rgb565, FB>>,
}

impl<FB> DmaTransfer for CompletedTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Buffer = FrameBuf<Rgb565, FB>;

    fn is_done(&self) -> bool {
        true
    }

    fn wait(mut self) -> FrameBuf<Rgb565, FB> {
        self.framebuffer
            .take()
            .expect("CompletedTransfer polled after completion")
    }
}

impl<FB> core::future::Future for CompletedTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Output = FrameBuf<Rgb565, FB>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let buf = unsafe { self.get_unchecked_mut() }
            .framebuffer
            .take()
            .expect("CompletedTransfer polled after completion");
        core::task::Poll::Ready(buf)
    }
}

impl<FB> AsyncDmaTransfer for CompletedTransfer<FB>
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type WaitFuture = Self;

    fn wait_async(self) -> Self::WaitFuture {
        self
    }
}

/// No-op display backend for simulators and testing.
pub struct SimulatorBackend;

impl SimulatorBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimulatorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize, const H: usize, FB> DisplayBackend<W, H, FB> for SimulatorBackend
where
    FB: DMACapableFrameBufferBackend<Color = Rgb565>,
{
    type Transfer = CompletedTransfer<FB>;

    fn start_dma_transfer(
        &mut self,
        framebuffer: FrameBuf<Rgb565, FB>,
    ) -> Result<CompletedTransfer<FB>, TransferError<FB>> {
        Ok(CompletedTransfer {
            framebuffer: Some(framebuffer),
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use core::cell::Cell;
    use embedded_graphics_core::pixelcolor::RgbColor;
    use embedded_graphics_framebuf::backends::EndianCorrectedBuffer;
    use std::vec;

    type TestBackend = EndianCorrectedBuffer<'static, Rgb565>;

    fn make_fb<const W: usize, const H: usize>() -> FrameBuf<Rgb565, TestBackend> {
        use embedded_graphics_framebuf::backends::EndianCorrection;
        let data: &'static mut [Rgb565] = vec![Rgb565::BLACK; W * H].leak();
        FrameBuf::new(
            EndianCorrectedBuffer::new(data, EndianCorrection::ToLittleEndian),
            W,
            H,
        )
    }

    struct RegionTransfer<FB: DMACapableFrameBufferBackend<Color = Rgb565>> {
        framebuffer: Option<FrameBuf<Rgb565, FB>>,
    }

    impl<FB: DMACapableFrameBufferBackend<Color = Rgb565>> DmaTransfer for RegionTransfer<FB> {
        type Buffer = FrameBuf<Rgb565, FB>;
        fn is_done(&self) -> bool {
            true
        }
        fn wait(mut self) -> FrameBuf<Rgb565, FB> {
            self.framebuffer.take().unwrap()
        }
    }

    impl<FB: DMACapableFrameBufferBackend<Color = Rgb565>> core::future::Future for RegionTransfer<FB> {
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

    impl<FB: DMACapableFrameBufferBackend<Color = Rgb565>> AsyncDmaTransfer for RegionTransfer<FB> {
        type WaitFuture = Self;
        fn wait_async(self) -> Self::WaitFuture {
            self
        }
    }

    struct RegionTrackingBackend {
        region_calls: Cell<usize>,
    }

    impl RegionTrackingBackend {
        fn new() -> Self {
            Self {
                region_calls: Cell::new(0),
            }
        }
    }

    impl<const W: usize, const H: usize, FB> DisplayBackend<W, H, FB> for RegionTrackingBackend
    where
        FB: DMACapableFrameBufferBackend<Color = Rgb565>,
    {
        type Transfer = RegionTransfer<FB>;

        fn start_dma_transfer(
            &mut self,
            framebuffer: FrameBuf<Rgb565, FB>,
        ) -> Result<RegionTransfer<FB>, TransferError<FB>> {
            Ok(RegionTransfer {
                framebuffer: Some(framebuffer),
            })
        }

        fn start_dma_transfer_region(
            &mut self,
            framebuffer: FrameBuf<Rgb565, FB>,
            _region: DisplayRegion,
        ) -> Result<RegionTransfer<FB>, TransferError<FB>> {
            self.region_calls.set(self.region_calls.get() + 1);
            Ok(RegionTransfer {
                framebuffer: Some(framebuffer),
            })
        }
    }

    #[test]
    fn test_simulator_backend_transfer_completes_immediately() {
        let mut backend = SimulatorBackend::new();
        let fb = make_fb::<2, 2>();
        let xfer = <SimulatorBackend as DisplayBackend<2, 2, TestBackend>>::start_dma_transfer(
            &mut backend,
            fb,
        )
        .unwrap();
        assert!(xfer.is_done());
        let _fb = xfer.wait();
    }

    #[test]
    fn test_region_tracking_backend_counts_region_transfers() {
        let mut backend = RegionTrackingBackend::new();
        let fb = make_fb::<2, 2>();
        let region = DisplayRegion::new(0, 0, 1, 1);
        let xfer = <RegionTrackingBackend as DisplayBackend<2, 2, TestBackend>>::start_dma_transfer_region(
            &mut backend,
            fb,
            region,
        )
        .unwrap();
        assert_eq!(backend.region_calls.get(), 1);
        let _ = xfer.wait();
    }
}
