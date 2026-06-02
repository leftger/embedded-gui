//! Animation timing helpers for embedded transitions.
//!
//! Provides interval remapping, table-based cubic easing samples, and the
//! moook spatial interpolation curve used for stack push/pop motion.

/// Normalized animation progress maximum (16-bit).
pub const NORMALIZED_MAX: i32 = 65_535;

/// Target frame interval at 30 Hz.
pub const FRAME_INTERVAL_MS: u32 = 33;

/// Default single-animation duration.
pub const DEFAULT_DURATION_MS: u32 = 250;

/// `PORT_HOLE_TRANSITION_DURATION_MS` / `ROUND_FLIP_ANIMATION_DURATION_MS`.
pub const PORT_HOLE_DURATION_MS: u32 = 6 * FRAME_INTERVAL_MS;

/// `SHUTTER_TRANSITION_DURATION_MS` (2 + 4 frames).
pub const SHUTTER_DURATION_MS: u32 = 6 * FRAME_INTERVAL_MS;

/// `interpolate_moook_duration()` (3 in + 4 out frames).
pub const MOOOK_DURATION_MS: u32 =
    (MOOOK_IN.len() as u32 + MOOOK_OUT.len() as u32) * FRAME_INTERVAL_MS;

const MOOOK_IN: [i32; 3] = [0, 1, 20];
const MOOOK_OUT: [i32; 4] = [4, 2, 1, 0];

/// Remap normalized progress into `[interval_start, interval_end]`.
#[inline]
pub fn timing_scaled(time_normalized: i32, interval_start: i32, interval_end: i32) -> i32 {
    if interval_end == interval_start {
        return NORMALIZED_MAX;
    }
    let result = time_normalized - interval_start;
    (result * NORMALIZED_MAX) / (interval_end - interval_start)
}

/// Clip normalized progress to `[0, NORMALIZED_MAX]`.
#[inline]
pub fn timing_clip(progress: i32) -> i32 {
    progress.clamp(0, NORMALIZED_MAX)
}

/// Two-phase helper: first half / second half of a transition (port-hole, shutter, round window).
#[inline]
pub fn timing_half_phase(progress: f32) -> (f32, bool) {
    if progress < 0.5 {
        (progress * 2.0, true)
    } else {
        ((progress - 0.5) * 2.0, false)
    }
}

/// Shutter timing: first 2/6 then 4/6 of total duration.
#[inline]
pub fn timing_shutter_phase(progress: f32) -> (f32, bool) {
    const FIRST: f32 = 2.0 / 6.0;
    if progress < FIRST {
        (progress / FIRST, true)
    } else {
        ((progress - FIRST) / (1.0 - FIRST), false)
    }
}

#[inline]
pub fn moook_in_duration_ms() -> u32 {
    MOOOK_IN.len() as u32 * FRAME_INTERVAL_MS
}

#[inline]
pub fn moook_out_duration_ms() -> u32 {
    MOOOK_OUT.len() as u32 * FRAME_INTERVAL_MS
}

#[inline]
pub fn moook_duration_ms() -> u32 {
    moook_in_duration_ms() + moook_out_duration_ms()
}

#[inline]
pub fn moook_soft_duration_ms(mid_frames: i32) -> u32 {
    moook_duration_ms() + mid_frames.max(0) as u32 * FRAME_INTERVAL_MS
}

fn interpolate_linear(normalized: i32, from: i64, to: i64) -> i64 {
    from + (normalized as i64 * (to - from)) / NORMALIZED_MAX as i64
}

fn interpolate_moook_frames(
    normalized: i32,
    from: i64,
    to: i64,
    frames_in: &[i32],
    frames_out: &[i32],
    mid_frames: i32,
    bounce_back: bool,
) -> i64 {
    let direction = if from == to {
        0
    } else if from < to {
        1
    } else {
        -1
    };
    if direction == 0 {
        return from;
    }
    let direction_out = if bounce_back { direction } else { -direction };
    let num_in = frames_in.len() as i32;
    let num_out = frames_out.len() as i32;
    let num_total = num_in + mid_frames + num_out;
    if num_total <= 0 {
        return if normalized >= NORMALIZED_MAX {
            to
        } else {
            from
        };
    }

    let mut frame_idx = ((normalized as i64 * num_total as i64
        + (NORMALIZED_MAX as i64 / (2 * num_total as i64)))
        / NORMALIZED_MAX as i64) as i32;
    frame_idx = frame_idx.clamp(0, num_total - 1);

    if normalized >= NORMALIZED_MAX {
        return to;
    }
    if frame_idx < 0 {
        return from;
    }
    if frame_idx < num_in {
        return from + direction as i64 * frames_in[frame_idx as usize] as i64;
    }
    if frame_idx < num_in + mid_frames && mid_frames > 0 {
        let shifted =
            normalized - ((num_in as i64 * NORMALIZED_MAX as i64) / num_total as i64) as i32;
        let mid_normalized = ((num_total as i64 * shifted as i64) / mid_frames as i64) as i32;
        let from_mid = from + direction as i64 * frames_in[(num_in - 1) as usize] as i64;
        let to_mid = to + direction_out as i64 * frames_out[0] as i64;
        return interpolate_linear(mid_normalized, from_mid, to_mid);
    }
    let out_idx = frame_idx - num_in - mid_frames;
    to + direction_out as i64 * frames_out[out_idx as usize] as i64
}

/// Full moook spatial interpolation (`interpolate_moook`).
pub fn interpolate_moook(normalized: i32, from: i64, to: i64) -> i64 {
    interpolate_moook_frames(normalized, from, to, &MOOOK_IN, &MOOOK_OUT, 0, true)
}

/// Moook with linear middle segment (`interpolate_moook_soft`).
pub fn interpolate_moook_soft(normalized: i32, from: i64, to: i64, mid_frames: i32) -> i64 {
    interpolate_moook_frames(
        normalized, from, to, &MOOOK_IN, &MOOOK_OUT, mid_frames, true,
    )
}

/// Map linear progress `t` in `[0, 1]` through moook spatial easing to `[0, 1]` (may overshoot).
pub fn moook_curve(t: f32) -> f32 {
    let normalized = (t.clamp(0.0, 1.0) * NORMALIZED_MAX as f32).round() as i32;
    let v = interpolate_moook(normalized, 0, NORMALIZED_MAX as i64);
    v as f32 / NORMALIZED_MAX as f32
}

/// Table-based cubic ease-in sample (32-entry lookup).
pub fn table_ease_in_sample(t: f32) -> f32 {
    const TABLE: [u16; 33] = [
        0, 64, 256, 576, 1024, 1600, 2304, 3136, 4096, 5184, 6400, 7744, 9216, 10816, 12544, 14400,
        16384, 18496, 20736, 23104, 25600, 28224, 30976, 33856, 36864, 40000, 43264, 46656, 50176,
        53824, 57600, 61504, 65535,
    ];
    ease_table_sample(t, &TABLE)
}

fn ease_table_sample(t: f32, table: &[u16]) -> f32 {
    if table.is_empty() {
        return t;
    }
    let progress = (t.clamp(0.0, 1.0) * NORMALIZED_MAX as f32).round() as i32;
    if progress <= 0 {
        return 0.0;
    }
    if progress >= NORMALIZED_MAX {
        return 1.0;
    }
    let max_entry = table.len() - 1;
    let stride = NORMALIZED_MAX / max_entry as i32;
    let index = (progress * max_entry as i32) / NORMALIZED_MAX;
    let from = table[index as usize] as i64;
    let delta = table[(index + 1) as usize] as i64 - from;
    let v = from + (delta * (progress - index * stride) as i64) / stride as i64;
    v as f32 / NORMALIZED_MAX as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moook_reaches_endpoints() {
        assert_eq!(interpolate_moook(0, 0, 100), 0);
        assert_eq!(interpolate_moook(NORMALIZED_MAX, 0, 100), 100);
    }

    #[test]
    fn timing_scaled_maps_interval() {
        let mid = timing_scaled(NORMALIZED_MAX / 2, 0, NORMALIZED_MAX);
        assert!((mid - NORMALIZED_MAX / 2).abs() <= 1);
    }

    #[test]
    fn moook_curve_is_monotonic_overall() {
        let a = moook_curve(0.0);
        let b = moook_curve(1.0);
        assert!((a - 0.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }
}
