//! Hardware-accelerated display scrolling support for MIPI DCS and ST7789-family display controllers.
//!
//! Provides zero-RAM hardware scrolling definitions, GRAM ring-buffer math, and dirty-rect calculation
//! for MCUs with tight SRAM constraints.

use crate::geometry::Rect;

/// Standard MIPI DCS vertical scroll command opcodes.
pub mod opcodes {
    /// Normal Display Mode On (cancels scroll mode if active).
    pub const NORON: u8 = 0x13;
    /// Vertical Scrolling Definition (configures TFA, VSA, BFA).
    pub const VSCRDEF: u8 = 0x33;
    /// Vertical Scroll Start Address (shifts display window in silicon).
    pub const VSCRSADD: u8 = 0x37;
}

/// Error returned when hardware scrolling parameters are invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollConfigError {
    /// The sum of top fixed area, scroll area, and bottom fixed area does not equal the physical height.
    HeightMismatch {
        tfa: u16,
        vsa: u16,
        bfa: u16,
        total: u16,
        expected: u16,
    },
    /// The scrolling area height is zero.
    ZeroScrollArea,
}

/// Configuration for MIPI DCS / ST7789 hardware vertical scrolling regions.
///
/// On controllers such as ST7789, ST7735, and ILI9341, the on-chip display RAM (GRAM) can be
/// partitioned into:
/// - **Top Fixed Area (TFA)**: Static scanlines at the top (e.g. status bar).
/// - **Vertical Scrolling Area (VSA)**: Circularly scrolling scanlines in the middle.
/// - **Bottom Fixed Area (BFA)**: Static scanlines at the bottom (e.g. navigation bar).
///
/// The constraint `TFA + VSA + BFA == physical_screen_height` must hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MipiDcsScrollConfig {
    pub tfa: u16,
    pub vsa: u16,
    pub bfa: u16,
}

/// Type alias for ST7789 vertical scroll configuration.
pub type St7789ScrollConfig = MipiDcsScrollConfig;

impl MipiDcsScrollConfig {
    /// Creates a full-screen vertical scrolling configuration (TFA = 0, BFA = 0).
    pub const fn full_screen(screen_height: u16) -> Self {
        Self {
            tfa: 0,
            vsa: screen_height,
            bfa: 0,
        }
    }

    /// Creates a vertical scrolling configuration with fixed top and bottom areas.
    pub const fn with_fixed_areas(
        tfa: u16,
        bfa: u16,
        screen_height: u16,
    ) -> Result<Self, ScrollConfigError> {
        let fixed = tfa.saturating_add(bfa);
        if fixed >= screen_height {
            return Err(ScrollConfigError::ZeroScrollArea);
        }
        let vsa = screen_height - fixed;
        if vsa == 0 {
            return Err(ScrollConfigError::ZeroScrollArea);
        }
        Ok(Self { tfa, vsa, bfa })
    }

    /// Total vertical scanlines defined by this configuration.
    #[inline]
    pub const fn total_lines(&self) -> u16 {
        self.tfa + self.vsa + self.bfa
    }

    /// Serializes the `VSCRDEF` (0x33) command and its 6 parameter bytes.
    ///
    /// Wire format:
    /// `[0x33, TFA[15..8], TFA[7..0], VSA[15..8], VSA[7..0], BFA[15..8], BFA[7..0]]`
    pub const fn vscrdef_command(&self) -> [u8; 7] {
        [
            opcodes::VSCRDEF,
            (self.tfa >> 8) as u8,
            (self.tfa & 0xFF) as u8,
            (self.vsa >> 8) as u8,
            (self.vsa & 0xFF) as u8,
            (self.bfa >> 8) as u8,
            (self.bfa & 0xFF) as u8,
        ]
    }

    /// Serializes the `VSCRSADD` (0x37) command and its 2 parameter bytes for a given scroll pointer.
    ///
    /// Wire format:
    /// `[0x37, VSP[15..8], VSP[7..0]]`
    pub const fn vscrsadd_command(&self, vsp: u16) -> [u8; 3] {
        [opcodes::VSCRSADD, (vsp >> 8) as u8, (vsp & 0xFF) as u8]
    }

    /// Serializes the `NORON` (0x13) Normal Display Mode On command.
    pub const fn noron_command() -> [u8; 1] {
        [opcodes::NORON]
    }
}

/// The damage report produced after a hardware scroll update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollDamage {
    /// The new vertical scroll pointer value written to `VSCRSADD` (0x37).
    pub new_vsp: u16,
    /// The rectangular region of the display that was newly exposed and needs redrawing.
    ///
    /// When `None`, the scroll shift did not reveal any new scanlines (delta was 0).
    pub exposed_damage: Option<Rect>,
    /// Whether the shift was large enough (>= VSA) that the entire scrolling area must be repainted.
    pub full_redraw_required: bool,
}

/// High-level controller managing hardware vertical scrolling and dirty region tracking.
///
/// Designed for low-RAM microcontrollers running Embassy or bare-metal loops where a full-screen
/// RAM framebuffer cannot fit. Hardware shifting moves display GRAM in silicon, while this controller
/// computes the exact rectangular strip that must be rendered via `render_dirty_buffered`.
#[derive(Clone, Copy, Debug)]
pub struct HardwareVerticalScroller {
    config: MipiDcsScrollConfig,
    width: u16,
    current_vsp: u16,
}

impl HardwareVerticalScroller {
    /// Creates a new hardware scroller with the given geometry and scroll configuration.
    pub const fn new(width: u16, config: MipiDcsScrollConfig) -> Self {
        Self {
            config,
            width,
            current_vsp: config.tfa,
        }
    }

    /// The display width in pixels.
    #[inline]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// The active scroll configuration.
    #[inline]
    pub const fn config(&self) -> &MipiDcsScrollConfig {
        &self.config
    }

    /// Current vertical scroll pointer (`VSCRSADD`) in display GRAM lines.
    #[inline]
    pub const fn current_vsp(&self) -> u16 {
        self.current_vsp
    }

    /// Resets the hardware scroll position back to default (unscrolled).
    pub fn reset(&mut self) -> ScrollDamage {
        let prev = self.current_vsp;
        self.current_vsp = self.config.tfa;
        if prev != self.current_vsp {
            ScrollDamage {
                new_vsp: self.current_vsp,
                exposed_damage: Some(Rect::new(
                    0,
                    self.config.tfa as i32,
                    self.width as u32,
                    self.config.vsa as u32,
                )),
                full_redraw_required: true,
            }
        } else {
            ScrollDamage {
                new_vsp: self.current_vsp,
                exposed_damage: None,
                full_redraw_required: false,
            }
        }
    }

    /// Computes the hardware scroll update when moving by `delta_y` scanlines.
    ///
    /// - Positive `delta_y`: scrolling downwards (content moves up, newly exposed scanlines appear at the bottom).
    /// - Negative `delta_y`: scrolling upwards (content moves down, newly exposed scanlines appear at the top).
    pub fn scroll_by(&mut self, delta_y: i16) -> ScrollDamage {
        if delta_y == 0 {
            return ScrollDamage {
                new_vsp: self.current_vsp,
                exposed_damage: None,
                full_redraw_required: false,
            };
        }

        let vsa = self.config.vsa as i32;
        let tfa = self.config.tfa as i32;

        if delta_y.unsigned_abs() >= self.config.vsa {
            // Delta exceeds entire scroll area: wrap pointer and require full redraw
            let offset = ((self.current_vsp as i32 - tfa + delta_y as i32).rem_euclid(vsa)) as u16;
            self.current_vsp = self.config.tfa + offset;
            return ScrollDamage {
                new_vsp: self.current_vsp,
                exposed_damage: Some(Rect::new(
                    0,
                    self.config.tfa as i32,
                    self.width as u32,
                    self.config.vsa as u32,
                )),
                full_redraw_required: true,
            };
        }

        let abs_delta = delta_y.unsigned_abs() as u32;
        let rel_vsp = (self.current_vsp as i32 - tfa + delta_y as i32).rem_euclid(vsa);
        self.current_vsp = self.config.tfa + rel_vsp as u16;

        let damage = if delta_y > 0 {
            // Scrolling down: content moves up, new slice appears at the bottom of the VSA.
            let damage_y = tfa + vsa - delta_y as i32;
            Rect::new(0, damage_y, self.width as u32, abs_delta)
        } else {
            // Scrolling up: content moves down, new slice appears at the top of the VSA.
            Rect::new(0, tfa, self.width as u32, abs_delta)
        };

        ScrollDamage {
            new_vsp: self.current_vsp,
            exposed_damage: Some(damage),
            full_redraw_required: false,
        }
    }

    /// Serializes the raw 3-byte `VSCRSADD` command for the current scroll position.
    #[inline]
    pub const fn current_vscrsadd_command(&self) -> [u8; 3] {
        self.config.vscrsadd_command(self.current_vsp)
    }
}

/// Optional trait implemented by displays or bus wrappers capable of direct hardware vertical scrolling.
pub trait HardwareVerticalScrollTarget {
    /// Error type returned by the display bus or controller.
    type Error;

    /// Configures the display's vertical scrolling definition (`VSCRDEF` 0x33).
    fn set_vertical_scroll_area(&mut self, config: &MipiDcsScrollConfig)
    -> Result<(), Self::Error>;

    /// Sets the vertical scroll start pointer (`VSCRSADD` 0x37).
    fn set_vertical_scroll_pointer(&mut self, vsp: u16) -> Result<(), Self::Error>;

    /// Cancels vertical scroll mode and returns to normal display mode (`NORON` 0x13).
    fn cancel_vertical_scroll(&mut self) -> Result<(), Self::Error>;
}
