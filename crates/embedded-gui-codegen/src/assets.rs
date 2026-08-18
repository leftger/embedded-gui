//! Conversion of source art into the forms `embedded-gui` draws.
//!
//! Both the `include_gui!` macro and the Studio preview call into here, which
//! is what keeps a design's preview and its firmware output byte-identical.
//! Nothing in this module decodes container formats: callers hand over raw
//! RGBA8 or file text, so the crate stays dependency-free.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetError {
    Bdf(String),
    Obj(String),
    Stl(String),
    UnsupportedMesh(String),
    Empty(&'static str),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::Bdf(msg) => write!(f, "invalid BDF font: {msg}"),
            AssetError::Obj(msg) => write!(f, "invalid OBJ mesh: {msg}"),
            AssetError::Stl(msg) => write!(f, "invalid STL mesh: {msg}"),
            AssetError::UnsupportedMesh(name) => {
                write!(f, "'{name}' is not a supported mesh format (.obj or .stl)")
            }
            AssetError::Empty(what) => write!(f, "{what} contained no usable data"),
        }
    }
}

impl std::error::Error for AssetError {}

/// A 1-bit-per-pixel bitmap, row-major and MSB-first, each row padded to a byte
/// boundary. Matches `embedded_gui::MonoBitmap`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonoBitmapData {
    pub width: u32,
    pub height: u32,
    pub bits: Vec<u8>,
}

impl MonoBitmapData {
    pub fn stride(&self) -> u32 {
        self.width.div_ceil(8)
    }
}

/// Threshold RGBA8 pixels into ink/paper.
///
/// A pixel is ink when it is opaque enough and its luminance falls on the
/// requested side of `threshold`. Art is usually black-on-white, so `invert`
/// defaults off in the KDL layer and flips for white-on-black sources.
pub fn mono_from_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    threshold: u8,
    invert: bool,
) -> MonoBitmapData {
    let stride = width.div_ceil(8);
    let mut bits = vec![0u8; (stride * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let Some(px) = rgba.get(idx..idx + 4) else {
                continue;
            };
            let alpha = px[3];
            let luminance =
                ((u16::from(px[0]) * 77 + u16::from(px[1]) * 150 + u16::from(px[2]) * 29) >> 8)
                    as u8;
            let dark = luminance < threshold;
            let ink = alpha >= 128 && (dark != invert);
            if ink {
                let byte = (y * stride + x / 8) as usize;
                bits[byte] |= 0x80 >> (x % 8);
            }
        }
    }
    MonoBitmapData {
        width,
        height,
        bits,
    }
}

/// A monospaced bitmap font, laid out as `embedded_gui::BitmapFont` expects:
/// glyphs stored back to back over the contiguous range starting at
/// `first_char`, each glyph `height * bytes_per_row` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitmapFontData {
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub line_height: u8,
    pub first_char: u8,
    pub bytes_per_row: u8,
    pub glyphs: Vec<u8>,
}

struct BdfGlyph {
    encoding: u32,
    /// Glyph bounding box: width, height, x offset, y offset from baseline.
    bbx: (i32, i32, i32, i32),
    dwidth: i32,
    rows: Vec<Vec<u8>>,
}

/// Parses a BDF font into a fixed-cell bitmap font.
///
/// Glyphs are composited into the font-wide bounding box so every cell shares
/// an origin, which is what makes the result safe to index by character code.
/// `chars`, when given, restricts the range that gets embedded: a shot counter
/// only needs digits, and dropping the rest saves flash.
pub fn parse_bdf(source: &str, chars: Option<&str>) -> Result<BitmapFontData, AssetError> {
    let mut font_bbox: Option<(i32, i32, i32, i32)> = None;
    let mut glyphs: BTreeMap<u32, BdfGlyph> = BTreeMap::new();

    let mut current: Option<BdfGlyph> = None;
    let mut in_bitmap = false;

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let keyword = parts.next().unwrap_or("");

        if in_bitmap {
            if keyword == "ENDCHAR" {
                in_bitmap = false;
                if let Some(glyph) = current.take() {
                    glyphs.insert(glyph.encoding, glyph);
                }
                continue;
            }
            let bytes =
                hex_row(line).ok_or_else(|| AssetError::Bdf(format!("bad BITMAP row '{line}'")))?;
            if let Some(glyph) = current.as_mut() {
                glyph.rows.push(bytes);
            }
            continue;
        }

        match keyword {
            "FONTBOUNDINGBOX" => {
                let v = read_ints(parts, 4)
                    .ok_or_else(|| AssetError::Bdf("malformed FONTBOUNDINGBOX".into()))?;
                font_bbox = Some((v[0], v[1], v[2], v[3]));
            }
            "STARTCHAR" => {
                current = Some(BdfGlyph {
                    encoding: 0,
                    bbx: (0, 0, 0, 0),
                    dwidth: 0,
                    rows: Vec::new(),
                });
            }
            "ENCODING" => {
                if let (Some(glyph), Some(code)) =
                    (current.as_mut(), parts.next().and_then(|v| v.parse().ok()))
                {
                    glyph.encoding = code;
                }
            }
            "DWIDTH" => {
                if let (Some(glyph), Some(w)) = (
                    current.as_mut(),
                    parts.next().and_then(|v| v.parse::<i32>().ok()),
                ) {
                    glyph.dwidth = w;
                }
            }
            "BBX" => {
                let v =
                    read_ints(parts, 4).ok_or_else(|| AssetError::Bdf("malformed BBX".into()))?;
                if let Some(glyph) = current.as_mut() {
                    glyph.bbx = (v[0], v[1], v[2], v[3]);
                }
            }
            "BITMAP" => in_bitmap = true,
            _ => {}
        }
    }

    let (fbb_w, fbb_h, ..) =
        font_bbox.ok_or_else(|| AssetError::Bdf("missing FONTBOUNDINGBOX".into()))?;
    if fbb_w <= 0 || fbb_h <= 0 {
        return Err(AssetError::Bdf("font bounding box has no area".into()));
    }

    let wanted: Option<Vec<u32>> = chars.map(|s| s.chars().map(|c| c as u32).collect());
    let selected: Vec<u32> = glyphs
        .keys()
        .copied()
        .filter(|code| *code <= 255 && wanted.as_ref().is_none_or(|list| list.contains(code)))
        .collect();
    if selected.is_empty() {
        return Err(AssetError::Empty("BDF font"));
    }

    // Size the cell to the glyphs actually being embedded rather than to
    // FONTBOUNDINGBOX. Display fonts reserve room for accents and descenders no
    // digit uses, and on a 96px-wide panel that padding is the difference
    // between two digits fitting and one.
    let mut left = i32::MAX;
    let mut right = i32::MIN;
    let mut bottom = i32::MAX;
    let mut top = i32::MIN;
    for code in &selected {
        let (gw, gh, gx, gy) = glyphs[code].bbx;
        left = left.min(gx);
        right = right.max(gx + gw);
        bottom = bottom.min(gy);
        top = top.max(gy + gh);
    }
    let cell_w = right - left;
    let cell_h = top - bottom;
    if cell_w <= 0 || cell_h <= 0 {
        return Err(AssetError::Empty("BDF font"));
    }
    if cell_w > 255 || cell_h > 255 {
        return Err(AssetError::Bdf(format!(
            "font cell {cell_w}x{cell_h} exceeds the 255px bitmap font limit"
        )));
    }

    let first = *selected.first().unwrap();
    let last = *selected.last().unwrap();
    let bytes_per_row = (cell_w as u32).div_ceil(8) as usize;
    let glyph_len = bytes_per_row * cell_h as usize;
    let mut out = vec![0u8; glyph_len * (last - first + 1) as usize];
    let mut advance = 0i32;

    for code in selected {
        let glyph = &glyphs[&code];
        advance = advance.max(if glyph.dwidth > 0 {
            glyph.dwidth
        } else {
            glyph.bbx.0
        });
        let (gw, gh, gx, gy) = glyph.bbx;
        // BDF y offsets are measured up from the baseline; our rows run down
        // from the cell top. Keeping every glyph relative to the shared cell
        // preserves how they line up on the baseline.
        let dx = gx - left;
        let dy = top - (gy + gh);
        let base = glyph_len * (code - first) as usize;
        for (row_idx, row) in glyph.rows.iter().enumerate() {
            let ty = dy + row_idx as i32;
            if ty < 0 || ty >= cell_h {
                continue;
            }
            for col in 0..gw {
                let byte = (col / 8) as usize;
                let bit = 7 - (col % 8);
                let set = row.get(byte).is_some_and(|b| b & (1 << bit) != 0);
                if !set {
                    continue;
                }
                let tx = dx + col;
                if tx < 0 || tx >= cell_w {
                    continue;
                }
                let idx = base + ty as usize * bytes_per_row + (tx / 8) as usize;
                out[idx] |= 0x80 >> (tx % 8);
            }
        }
    }

    Ok(BitmapFontData {
        width: cell_w as u8,
        height: cell_h as u8,
        advance: advance.clamp(1, 255) as u8,
        line_height: cell_h as u8,
        first_char: first as u8,
        bytes_per_row: bytes_per_row as u8,
        glyphs: out,
    })
}

fn hex_row(line: &str) -> Option<Vec<u8>> {
    if !line.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(line.len() / 2);
    for pair in line.as_bytes().chunks(2) {
        let s = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(s, 16).ok()?);
    }
    Some(bytes)
}

fn read_ints<'a>(parts: impl Iterator<Item = &'a str>, count: usize) -> Option<Vec<i32>> {
    let values: Vec<i32> = parts.take(count).filter_map(|v| v.parse().ok()).collect();
    (values.len() == count).then_some(values)
}

/// Triangle mesh in the layout `embedded_3dgfx::Geometry` expects.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
    /// One normal per face, for flat-shaded lighting.
    pub normals: Vec<[f32; 3]>,
}

/// Parses the subset of Wavefront OBJ that matters for decorative geometry:
/// `v` positions and `f` faces, with polygons fan-triangulated.
pub fn parse_obj(source: &str) -> Result<MeshData, AssetError> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let coords: Vec<f32> = parts.take(3).filter_map(|v| v.parse().ok()).collect();
                if coords.len() != 3 {
                    return Err(AssetError::Obj(format!("malformed vertex '{line}'")));
                }
                vertices.push([coords[0], coords[1], coords[2]]);
            }
            Some("f") => {
                let indices: Vec<usize> = parts
                    .filter_map(|token| {
                        let first = token.split('/').next()?;
                        let idx: i64 = first.parse().ok()?;
                        // OBJ indices are 1-based; negatives count back from the end.
                        let resolved = if idx < 0 {
                            vertices.len() as i64 + idx
                        } else {
                            idx - 1
                        };
                        (resolved >= 0 && (resolved as usize) < vertices.len())
                            .then_some(resolved as usize)
                    })
                    .collect();
                for i in 1..indices.len().saturating_sub(1) {
                    faces.push([indices[0], indices[i], indices[i + 1]]);
                }
            }
            _ => {}
        }
    }

    if vertices.is_empty() || faces.is_empty() {
        return Err(AssetError::Empty("OBJ mesh"));
    }

    let mut mesh = MeshData {
        vertices,
        faces,
        normals: Vec::new(),
    };
    mesh.recompute_normals();
    Ok(mesh)
}

/// Parses a mesh from file bytes, choosing the reader by extension.
///
/// CAD tools export STL and modelling tools export OBJ, so a project can point
/// a `mesh` node at whichever file the artist already has.
pub fn parse_mesh(source_name: &str, bytes: &[u8]) -> Result<MeshData, AssetError> {
    let extension = source_name
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "obj" => {
            let text = std::str::from_utf8(bytes)
                .map_err(|err| AssetError::Obj(format!("not UTF-8 text: {err}")))?;
            parse_obj(text)
        }
        "stl" => parse_stl(bytes),
        _ => Err(AssetError::UnsupportedMesh(source_name.to_string())),
    }
}

/// Parses binary or ASCII STL.
///
/// STL repeats full coordinates per triangle corner, so identical positions are
/// welded back into shared vertices; the 3D rasterizer transforms each vertex
/// once, and a logo exported from CAD typically sheds two thirds of its list.
pub fn parse_stl(bytes: &[u8]) -> Result<MeshData, AssetError> {
    let triangles = if is_ascii_stl(bytes) {
        let text = std::str::from_utf8(bytes)
            .map_err(|err| AssetError::Stl(format!("not UTF-8 text: {err}")))?;
        parse_ascii_stl(text)?
    } else {
        parse_binary_stl(bytes)?
    };

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();
    let mut lookup: BTreeMap<[u32; 3], usize> = BTreeMap::new();

    for corners in triangles {
        let mut face = [0usize; 3];
        for (slot, corner) in corners.iter().enumerate() {
            let key = [
                corner[0].to_bits(),
                corner[1].to_bits(),
                corner[2].to_bits(),
            ];
            face[slot] = *lookup.entry(key).or_insert_with(|| {
                vertices.push(*corner);
                vertices.len() - 1
            });
        }
        // Drop degenerate triangles: they carry no surface and would only
        // produce a zero-length normal.
        if face[0] != face[1] && face[1] != face[2] && face[0] != face[2] {
            faces.push(face);
        }
    }

    if vertices.is_empty() || faces.is_empty() {
        return Err(AssetError::Empty("STL mesh"));
    }

    let mut mesh = MeshData {
        vertices,
        faces,
        normals: Vec::new(),
    };
    mesh.recompute_normals();
    Ok(mesh)
}

fn is_ascii_stl(bytes: &[u8]) -> bool {
    // The 80-byte binary header may itself start with "solid", so trust the
    // triangle count instead: a binary file is exactly 84 + 50 * count bytes.
    if bytes.len() >= 84 {
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        if bytes.len() == 84 + count * 50 {
            return false;
        }
    }
    bytes.starts_with(b"solid")
}

fn parse_binary_stl(bytes: &[u8]) -> Result<Vec<[[f32; 3]; 3]>, AssetError> {
    if bytes.len() < 84 {
        return Err(AssetError::Stl("file is shorter than the header".into()));
    }
    let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
    let expected = 84 + count * 50;
    if bytes.len() < expected {
        return Err(AssetError::Stl(format!(
            "header claims {count} triangles but the file holds {} bytes",
            bytes.len()
        )));
    }

    let mut triangles = Vec::with_capacity(count);
    for index in 0..count {
        // Each record is a face normal we recompute anyway, then 3 corners.
        let base = 84 + index * 50 + 12;
        let mut corners = [[0.0f32; 3]; 3];
        for (corner_idx, corner) in corners.iter_mut().enumerate() {
            for (axis, value) in corner.iter_mut().enumerate() {
                let at = base + corner_idx * 12 + axis * 4;
                *value =
                    f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
            }
        }
        triangles.push(corners);
    }
    Ok(triangles)
}

fn parse_ascii_stl(source: &str) -> Result<Vec<[[f32; 3]; 3]>, AssetError> {
    let mut triangles = Vec::new();
    let mut corners: Vec<[f32; 3]> = Vec::new();

    for line in source.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("vertex") => {
                let coords: Vec<f32> = parts.take(3).filter_map(|v| v.parse().ok()).collect();
                if coords.len() != 3 {
                    return Err(AssetError::Stl(format!(
                        "malformed vertex '{}'",
                        line.trim()
                    )));
                }
                corners.push([coords[0], coords[1], coords[2]]);
            }
            Some("endloop") => {
                if corners.len() == 3 {
                    triangles.push([corners[0], corners[1], corners[2]]);
                }
                corners.clear();
            }
            _ => {}
        }
    }
    Ok(triangles)
}

impl MeshData {
    pub fn recompute_normals(&mut self) {
        self.normals.clear();
        self.normals.reserve(self.faces.len());
        for [a, b, c] in self.faces.iter().copied() {
            let (va, vb, vc) = (self.vertices[a], self.vertices[b], self.vertices[c]);
            let u = [vb[0] - va[0], vb[1] - va[1], vb[2] - va[2]];
            let v = [vc[0] - va[0], vc[1] - va[1], vc[2] - va[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            self.normals.push(if len > f32::EPSILON {
                [n[0] / len, n[1] / len, n[2] / len]
            } else {
                [0.0, 0.0, 1.0]
            });
        }
    }

    /// Centers the mesh on the origin and scales it to unit radius, so the
    /// `scale` authored in KDL means the same thing for any source model.
    pub fn normalize(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for v in &self.vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(v[axis]);
                max[axis] = max[axis].max(v[axis]);
            }
        }
        let center = [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ];
        let mut radius = 0.0f32;
        for v in &self.vertices {
            let d = [v[0] - center[0], v[1] - center[1], v[2] - center[2]];
            radius = radius.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
        if radius <= f32::EPSILON {
            return;
        }
        for v in &mut self.vertices {
            for axis in 0..3 {
                v[axis] = (v[axis] - center[axis]) / radius;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_rgba_into_msb_first_rows() {
        // 2x1 black/white pixels, both opaque.
        let rgba = [0, 0, 0, 255, 255, 255, 255, 255];
        let mono = mono_from_rgba(2, 1, &rgba, 128, false);
        assert_eq!(mono.bits, vec![0b1000_0000]);
        let inverted = mono_from_rgba(2, 1, &rgba, 128, true);
        assert_eq!(inverted.bits, vec![0b0100_0000]);
    }

    #[test]
    fn parses_bdf_into_fixed_cells() {
        let bdf = "\
STARTFONT 2.1
FONTBOUNDINGBOX 8 8 0 0
CHARS 2
STARTCHAR zero
ENCODING 48
DWIDTH 8 0
BBX 8 2 0 6
BITMAP
FF
81
ENDCHAR
STARTCHAR one
ENCODING 49
DWIDTH 8 0
BBX 8 1 0 7
BITMAP
18
ENDCHAR
ENDFONT
";
        let font = parse_bdf(bdf, None).unwrap();
        // The cell crops to the two glyphs, not to the 8x8 FONTBOUNDINGBOX.
        assert_eq!((font.width, font.height, font.bytes_per_row), (8, 2, 1));
        assert_eq!(font.first_char, b'0');
        assert_eq!(font.advance, 8);
        assert_eq!(font.glyphs.len(), 4);
        // '0' fills both rows of the cell.
        assert_eq!(font.glyphs[0], 0xFF);
        assert_eq!(font.glyphs[1], 0x81);
        // '1' sits a row higher on the baseline, so it lands on the cell top.
        assert_eq!(font.glyphs[2], 0x18);
        assert_eq!(font.glyphs[3], 0x00);
    }

    #[test]
    fn restricts_bdf_glyphs_to_requested_chars() {
        let bdf = "\
FONTBOUNDINGBOX 8 8 0 0
STARTCHAR a
ENCODING 65
DWIDTH 8 0
BBX 8 1 0 7
BITMAP
FF
ENDCHAR
STARTCHAR b
ENCODING 66
DWIDTH 8 0
BBX 8 1 0 7
BITMAP
0F
ENDCHAR
";
        let font = parse_bdf(bdf, Some("B")).unwrap();
        assert_eq!(font.first_char, b'B');
        assert_eq!((font.width, font.height), (8, 1));
        assert_eq!(font.glyphs.len(), 1);
        assert_eq!(font.glyphs[0], 0x0F);
    }

    #[test]
    fn welds_shared_corners_from_binary_stl() {
        // Two triangles forming a quad, so four of the six corners are shared.
        let corners: [[[f32; 3]; 3]; 2] = [
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
            [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        ];
        let mut bytes = vec![0u8; 80];
        bytes.extend_from_slice(&2u32.to_le_bytes());
        for triangle in corners {
            bytes.extend_from_slice(&[0u8; 12]);
            for corner in triangle {
                for axis in corner {
                    bytes.extend_from_slice(&axis.to_le_bytes());
                }
            }
            bytes.extend_from_slice(&[0u8; 2]);
        }

        let mesh = parse_mesh("logo.stl", &bytes).unwrap();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.faces, vec![[0, 1, 2], [0, 2, 3]]);
        assert_eq!(mesh.normals.len(), 2);
    }

    #[test]
    fn reads_ascii_stl_and_rejects_unknown_extensions() {
        let stl = "\
solid demo
facet normal 0 0 1
  outer loop
    vertex 0 0 0
    vertex 1 0 0
    vertex 0 1 0
  endloop
endfacet
endsolid demo
";
        let mesh = parse_mesh("demo.stl", stl.as_bytes()).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces, vec![[0, 1, 2]]);

        assert!(matches!(
            parse_mesh("demo.ply", stl.as_bytes()),
            Err(AssetError::UnsupportedMesh(_))
        ));
    }

    #[test]
    fn triangulates_obj_quads_and_normalizes() {
        let obj = "\
v -1 -1 0
v 1 -1 0
v 1 1 0
v -1 1 0
f 1 2 3 4
";
        let mut mesh = parse_obj(obj).unwrap();
        assert_eq!(mesh.faces, vec![[0, 1, 2], [0, 2, 3]]);
        assert_eq!(mesh.normals.len(), 2);
        assert!((mesh.normals[0][2] - 1.0).abs() < 1e-5);
        mesh.normalize();
        let radius = mesh
            .vertices
            .iter()
            .map(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
            .fold(0.0f32, f32::max);
        assert!((radius - 1.0).abs() < 1e-5);
    }
}
