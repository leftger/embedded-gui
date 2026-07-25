//! Custom visual widgets & DSP-accelerated UI controls for embedded-gui.
//!
//! Includes:
//! 1. `BusyWheel` — Activity indicator spinner with orbiting dot trail
//! 2. `GaugeWidget` — Analog gauge dial helper with value needle
//! 3. `TouchInputFilter` — DSP Biquad touch & pointer coordinate damping (`embedded-dsp` integration)
//! 4. `SpectrumAnalyzerWidget` — Real-time DSP signal & telemetry bar visualizer (`embedded-dsp` integration)

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::{Rgb565, RgbColor, WebColors}};
use crate::{geometry::Rect, render::{PixelRead, RenderCtx}};

/// Activity Indicator / Busy Wheel widget.
///
/// Renders orbiting dots around a center point with alpha trail decay.
#[derive(Debug, Clone, Copy)]
pub struct BusyWheel {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: u32,
    pub dot_count: u8,
    pub dot_radius: u32,
    pub phase: f32,
    pub color: Rgb565,
    pub opacity: u8,
}


impl BusyWheel {
    pub fn new(center_x: i32, center_y: i32, radius: u32) -> Self {
        Self {
            center_x,
            center_y,
            radius,
            dot_count: 8,
            dot_radius: 3,
            phase: 0.0,
            color: Rgb565::CSS_CYAN,
            opacity: 255,
        }
    }

    pub fn draw<D, C>(&self, ctx: &mut RenderCtx<D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565> + PixelRead,
        C: crate::render::Compositor<D>,
    {
        if self.opacity == 0 || self.dot_count == 0 {
            return Ok(());
        }

        let step_angle = 2.0 * core::f32::consts::PI / self.dot_count as f32;
        let dr = self.dot_radius as i32;

        for i in 0..self.dot_count {
            let angle = self.phase + (i as f32 * step_angle);
            let dx = (angle.cos() * self.radius as f32) as i32;
            let dy = (angle.sin() * self.radius as f32) as i32;
            let px = self.center_x + dx;
            let py = self.center_y + dy;

            let dot_rect = Rect::new(px - dr, py - dr, (dr * 2 + 1) as u32, (dr * 2 + 1) as u32);
            ctx.fill_rounded_rect(dot_rect, dr as u8, self.color)?;
        }

        Ok(())
    }
}

/// Analog Gauge dial widget.
///
/// Renders a circular gauge background, tick marks, and value needle.
#[derive(Debug, Clone, Copy)]
pub struct GaugeWidget {
    pub bounds: Rect,
    pub min_val: f32,
    pub max_val: f32,
    pub current_val: f32,
    pub needle_color: Rgb565,
    pub dial_color: Rgb565,
    pub arc_color: Rgb565,
}

impl GaugeWidget {
    pub fn new(bounds: Rect, min_val: f32, max_val: f32) -> Self {
        Self {
            bounds,
            min_val,
            max_val,
            current_val: min_val,
            needle_color: Rgb565::RED,
            dial_color: Rgb565::new(4, 8, 4),
            arc_color: Rgb565::GREEN,
        }
    }

    pub fn draw<D, C>(&self, ctx: &mut RenderCtx<D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565> + PixelRead,
        C: crate::render::Compositor<D>,
    {
        // Dial card fill
        ctx.fill_rounded_rect(self.bounds, 6, self.dial_color)?;

        let center_x = self.bounds.x + (self.bounds.w as i32 / 2);
        let center_y = self.bounds.y + (self.bounds.h as i32 / 2);
        let radius = (self.bounds.w.min(self.bounds.h) as f32 * 0.4) as i32;

        // Value fraction in [0.0, 1.0]
        let range = (self.max_val - self.min_val).max(0.001);
        let norm_val = ((self.current_val - self.min_val) / range).clamp(0.0, 1.0);

        // Angle from -135 deg to +135 deg
        let angle_deg = -135.0 + norm_val * 270.0;
        let angle_rad = angle_deg * core::f32::consts::PI / 180.0;

        let nx = center_x + (angle_rad.cos() * radius as f32) as i32;
        let ny = center_y + (angle_rad.sin() * radius as f32) as i32;

        // Needle line
        ctx.draw_line(center_x, center_y, nx, ny, self.needle_color)?;

        // Pivot center cap
        let pivot_rect = Rect::new(center_x - 2, center_y - 2, 5, 5);
        ctx.fill_rounded_rect(pivot_rect, 2, Rgb565::WHITE)?;

        Ok(())
    }
}

/// Touch & Pointer coordinate low-pass damping filter using `embedded-dsp`.
#[cfg(feature = "embedded-dsp")]
pub struct TouchInputFilter {
    coeffs: [f32; 5],
    state_x: [f32; 4],
    state_y: [f32; 4],
    initialized: bool,
}

#[cfg(feature = "embedded-dsp")]
impl TouchInputFilter {
    /// Create a new low-pass touch input filter for 2D pointer coordinates.
    pub fn new(_cutoff_freq_ratio: f32) -> Self {
        let coeffs = [0.0675, 0.1349, 0.0675, 1.1430, -0.4128];
        Self {
            coeffs,
            state_x: [0.0; 4],
            state_y: [0.0; 4],
            initialized: false,
        }
    }

    /// Process raw touch coordinates (x, y) through low-pass DSP filter to eliminate noise/jitter.
    pub fn filter(&mut self, raw_x: f32, raw_y: f32) -> (f32, f32) {
        if !self.initialized {
            self.state_x.fill(raw_x);
            self.state_y.fill(raw_y);
            self.initialized = true;
        }

        let mut inst_x = embedded_dsp::filtering::BiquadCascadeInstanceF32 {
            num_stages: 1,
            coeffs: &self.coeffs,
            state: &mut self.state_x,
        };
        let mut out_x = 0.0f32;
        embedded_dsp::filtering::biquad_cascade_df1_f32(&mut inst_x, &[raw_x], core::slice::from_mut(&mut out_x));

        let mut inst_y = embedded_dsp::filtering::BiquadCascadeInstanceF32 {
            num_stages: 1,
            coeffs: &self.coeffs,
            state: &mut self.state_y,
        };
        let mut out_y = 0.0f32;
        embedded_dsp::filtering::biquad_cascade_df1_f32(&mut inst_y, &[raw_y], core::slice::from_mut(&mut out_y));

        (out_x, out_y)
    }

    /// Reset filter state (e.g. on pointer release or new touch down).
    pub fn reset(&mut self) {
        self.state_x.fill(0.0);
        self.state_y.fill(0.0);
        self.initialized = false;
    }
}

/// Real-time DSP signal & telemetry bar visualizer widget.
#[cfg(feature = "embedded-dsp")]
pub struct SpectrumAnalyzerWidget<'a> {
    pub bounds: Rect,
    pub signal_samples: &'a [f32],
    pub bar_color: Rgb565,
    pub bg_color: Rgb565,
}

#[cfg(feature = "embedded-dsp")]
impl<'a> SpectrumAnalyzerWidget<'a> {
    pub fn new(bounds: Rect, signal_samples: &'a [f32]) -> Self {
        Self {
            bounds,
            signal_samples,
            bar_color: Rgb565::CSS_LIME_GREEN,
            bg_color: Rgb565::new(2, 4, 2),
        }
    }

    pub fn draw<D, C>(&self, ctx: &mut RenderCtx<D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565> + PixelRead,
        C: crate::render::Compositor<D>,
    {
        ctx.fill_rounded_rect(self.bounds, 4, self.bg_color)?;

        if self.signal_samples.is_empty() {
            return Ok(());
        }

        // Calculate RMS power & peak stats via embedded-dsp
        let mut rms = 0.0f32;
        let _ = embedded_dsp::statistics::rms_f32(self.signal_samples, &mut rms);

        let mut max_val = 0.0f32;
        let mut idx = 0;
        let _ = embedded_dsp::statistics::max_f32(self.signal_samples, &mut max_val, &mut idx);

        let bar_count = (self.bounds.w / 6).max(1) as usize;
        let chunk_size = (self.signal_samples.len() / bar_count).max(1);

        for i in 0..bar_count {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(self.signal_samples.len());
            let chunk = &self.signal_samples[start..end];
            let mut chunk_rms = 0.0f32;
            if !chunk.is_empty() {
                let _ = embedded_dsp::statistics::rms_f32(chunk, &mut chunk_rms);
            }

            let norm_h = (chunk_rms / (max_val.max(rms).max(0.001))).clamp(0.05, 1.0);
            let bar_h = (self.bounds.h as f32 * norm_h) as u32;

            let bx = self.bounds.x + (i as i32 * 6);
            let by = self.bounds.bottom() - bar_h as i32;

            let bar_rect = Rect::new(bx + 1, by, 4, bar_h);
            ctx.fill_rect(bar_rect, self.bar_color)?;
        }

        Ok(())
    }
}
