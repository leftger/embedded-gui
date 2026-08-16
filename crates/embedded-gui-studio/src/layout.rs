//! Grid layout track calculation helpers for simulated 2D viewports.

use embedded_gui_codegen::GridTrackDef;

/// Resolves fractional (`fr`) and fixed (`px`) track constraints against available container size.
pub fn compute_track_sizes(tracks: &[GridTrackDef], available_size: f32, gap: f32) -> Vec<f32> {
    let n = tracks.len();
    if n == 0 {
        return vec![available_size];
    }
    let total_gap = gap * (n.saturating_sub(1) as f32);
    let net_space = (available_size - total_gap).max(0.0);

    let mut fixed_sum = 0.0;
    let mut total_fr = 0u32;

    for t in tracks {
        match t {
            GridTrackDef::Px(px) => fixed_sum += *px as f32,
            GridTrackDef::Fr(fr) => total_fr += *fr as u32,
            GridTrackDef::Auto => fixed_sum += 32.0,
        }
    }

    let remaining = (net_space - fixed_sum).max(0.0);
    let fr_unit = if total_fr > 0 {
        remaining / (total_fr as f32)
    } else {
        0.0
    };

    tracks
        .iter()
        .map(|t| match t {
            GridTrackDef::Px(px) => *px as f32,
            GridTrackDef::Fr(fr) => (*fr as f32) * fr_unit,
            GridTrackDef::Auto => 32.0,
        })
        .collect()
}
