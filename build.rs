use std::{collections::HashMap, env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let out_file = out_dir.join("generated_ascii_3x5.rs");
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir missing"));
    let assets_dir = manifest_dir.join("assets").join("fonts");
    let map_3x5 = load_glyph_overrides(&assets_dir.join("ascii_3x5.txt"));
    let map_4x7 = load_glyph_overrides(&assets_dir.join("ascii_4x7.txt"));

    let mut src = String::from("pub static ASCII_3X5_GLYPHS: [[u8; 5]; 95] = [\n");
    for code in 32u8..=126u8 {
        let ch = code as char;
        let g = map_3x5.get(&ch).copied().unwrap_or_else(|| glyph_3x5(ch));
        src.push_str(&format!(
            "    [{:#05b}, {:#05b}, {:#05b}, {:#05b}, {:#05b}],\n",
            g[0], g[1], g[2], g[3], g[4]
        ));
    }
    src.push_str("];\n");
    src.push_str("\n");
    src.push_str("pub static ASCII_4X7_GLYPHS: [[u8; 5]; 95] = [\n");
    for code in 32u8..=126u8 {
        let ch = code as char;
        let g = map_4x7.get(&ch).copied().unwrap_or_else(|| glyph_3x5(ch));
        src.push_str(&format!(
            "    [{:#05b}, {:#05b}, {:#05b}, {:#05b}, {:#05b}],\n",
            g[0], g[1], g[2], g[3], g[4]
        ));
    }
    src.push_str("];\n");

    fs::write(out_file, src).expect("failed writing generated ascii font");
}

fn load_glyph_overrides(path: &PathBuf) -> HashMap<char, [u8; 5]> {
    let mut out = HashMap::new();
    let Ok(contents) = fs::read_to_string(path) else {
        return out;
    };

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rows)) = line.split_once(':') else {
            continue;
        };
        let Some(ch) = parse_key(key.trim()) else {
            continue;
        };
        let mut glyph = [0u8; 5];
        let mut ok = true;
        let parts: Vec<&str> = rows.split(',').collect();
        if parts.len() != 5 {
            continue;
        }
        for (idx, part) in parts.into_iter().enumerate() {
            let row = part.trim();
            if row.len() != 3 || row.chars().any(|c| c != '0' && c != '1') {
                ok = false;
                break;
            }
            let mut bits = 0u8;
            for c in row.chars() {
                bits = (bits << 1) | u8::from(c == '1');
            }
            glyph[idx] = bits;
        }
        if ok {
            out.insert(ch, glyph);
        }
    }

    out
}

fn parse_key(key: &str) -> Option<char> {
    if key.eq_ignore_ascii_case("space") {
        return Some(' ');
    }
    if key.chars().count() == 1 {
        return key.chars().next();
    }
    None
}

fn glyph_3x5(ch: char) -> [u8; 5] {
    match ch.to_ascii_uppercase() {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b111, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b110, 0b101, 0b010],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b010, 0b101, 0b010, 0b101, 0b010],
        '9' => [0b010, 0b101, 0b011, 0b001, 0b110],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '_' => [0b000, 0b000, 0b000, 0b000, 0b111],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        ' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
        _ => [0b111, 0b101, 0b101, 0b101, 0b111],
    }
}
