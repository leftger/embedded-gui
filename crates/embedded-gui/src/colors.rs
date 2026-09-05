//! Standard CSS color constants and RGB565 conversion utilities.
//!
//! Provides all 148 standard CSS named colors mapped directly to [`Rgb565`].

use embedded_graphics_core::pixelcolor::Rgb565;

/// Converts a 16-bit packed RGB565 raw value into [`Rgb565`].
#[inline]
pub const fn from_raw565(raw: u16) -> Rgb565 {
    Rgb565::new(
        ((raw >> 11) & 0x1F) as u8,
        ((raw >> 5) & 0x3F) as u8,
        (raw & 0x1F) as u8,
    )
}

/// Converts 24-bit RGB888 components into [`Rgb565`].
#[inline]
pub const fn rgb888(r: u8, g: u8, b: u8) -> Rgb565 {
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

/// Converts a 24-bit hex RGB value (e.g. `0xFF5733`) into [`Rgb565`].
#[inline]
pub const fn from_hex24(hex: u32) -> Rgb565 {
    rgb888(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

// ---------------------------------------------------------------------------
// CSS Color Constants
// ---------------------------------------------------------------------------
pub const ALICEBLUE: Rgb565 = from_raw565(0xefbf);
pub const ANTIQUEWHITE: Rgb565 = from_raw565(0xf75a);
pub const AQUA: Rgb565 = from_raw565(0x07ff);
pub const AQUAMARINE: Rgb565 = from_raw565(0x7ffa);
pub const AZURE: Rgb565 = from_raw565(0xefff);
pub const BEIGE: Rgb565 = from_raw565(0xf7bb);
pub const BISQUE: Rgb565 = from_raw565(0xff18);
pub const BLACK: Rgb565 = from_raw565(0x0000);
pub const BLANCHEDALMOND: Rgb565 = from_raw565(0xff59);
pub const BLUE: Rgb565 = from_raw565(0x001f);
pub const BLUEVIOLET: Rgb565 = from_raw565(0x897b);
pub const BROWN: Rgb565 = from_raw565(0xa145);
pub const BURLYWOOD: Rgb565 = from_raw565(0xddb0);
pub const CADETBLUE: Rgb565 = from_raw565(0x64f3);
pub const CHARTREUSE: Rgb565 = from_raw565(0x7fe0);
pub const CHOCOLATE: Rgb565 = from_raw565(0xd344);
pub const CORAL: Rgb565 = from_raw565(0xfbea);
pub const CORNFLOWERBLUE: Rgb565 = from_raw565(0x64bd);
pub const CORNSILK: Rgb565 = from_raw565(0xffbb);
pub const CRIMSON: Rgb565 = from_raw565(0xd8a7);
pub const CYAN: Rgb565 = from_raw565(0x07ff);
pub const DARKBLUE: Rgb565 = from_raw565(0x0011);
pub const DARKCYAN: Rgb565 = from_raw565(0x0451);
pub const DARKGOLDENROD: Rgb565 = from_raw565(0xb421);
pub const DARKGRAY: Rgb565 = from_raw565(0xad55);
pub const DARKGREEN: Rgb565 = from_raw565(0x0320);
pub const DARKGREY: Rgb565 = from_raw565(0xad55);
pub const DARKKHAKI: Rgb565 = from_raw565(0xbdad);
pub const DARKMAGENTA: Rgb565 = from_raw565(0x8811);
pub const DARKOLIVEGREEN: Rgb565 = from_raw565(0x5346);
pub const DARKORANGE: Rgb565 = from_raw565(0xfc60);
pub const DARKORCHID: Rgb565 = from_raw565(0x9999);
pub const DARKRED: Rgb565 = from_raw565(0x8800);
pub const DARKSALMON: Rgb565 = from_raw565(0xe4af);
pub const DARKSEAGREEN: Rgb565 = from_raw565(0x8dd1);
pub const DARKSLATEBLUE: Rgb565 = from_raw565(0x49f1);
pub const DARKSLATEGRAY: Rgb565 = from_raw565(0x328a);
pub const DARKSLATEGREY: Rgb565 = from_raw565(0x328a);
pub const DARKTURQUOISE: Rgb565 = from_raw565(0x0679);
pub const DARKVIOLET: Rgb565 = from_raw565(0x901a);
pub const DEEPPINK: Rgb565 = from_raw565(0xf8b2);
pub const DEEPSKYBLUE: Rgb565 = from_raw565(0x05ff);
pub const DIMGRAY: Rgb565 = from_raw565(0x6b4d);
pub const DIMGREY: Rgb565 = from_raw565(0x6b4d);
pub const DODGERBLUE: Rgb565 = from_raw565(0x249f);
pub const FIREBRICK: Rgb565 = from_raw565(0xb104);
pub const FLORALWHITE: Rgb565 = from_raw565(0xffdd);
pub const FORESTGREEN: Rgb565 = from_raw565(0x2444);
pub const FUCHSIA: Rgb565 = from_raw565(0xf81f);
pub const GAINSBORO: Rgb565 = from_raw565(0xdedb);
pub const GHOSTWHITE: Rgb565 = from_raw565(0xf7bf);
pub const GOLD: Rgb565 = from_raw565(0xfea0);
pub const GOLDENROD: Rgb565 = from_raw565(0xdd24);
pub const GRAY: Rgb565 = from_raw565(0x8410);
pub const GREEN: Rgb565 = from_raw565(0x0400);
pub const GREENYELLOW: Rgb565 = from_raw565(0xafe6);
pub const GREY: Rgb565 = from_raw565(0x8410);
pub const HONEYDEW: Rgb565 = from_raw565(0xeffd);
pub const HOTPINK: Rgb565 = from_raw565(0xfb56);
pub const INDIANRED: Rgb565 = from_raw565(0xcaeb);
pub const INDIGO: Rgb565 = from_raw565(0x4810);
pub const IVORY: Rgb565 = from_raw565(0xfffd);
pub const KHAKI: Rgb565 = from_raw565(0xef31);
pub const LAVENDER: Rgb565 = from_raw565(0xe73e);
pub const LAVENDERBLUSH: Rgb565 = from_raw565(0xff7e);
pub const LAWNGREEN: Rgb565 = from_raw565(0x7fc0);
pub const LEMONCHIFFON: Rgb565 = from_raw565(0xffd9);
pub const LIGHTBLUE: Rgb565 = from_raw565(0xaebc);
pub const LIGHTCORAL: Rgb565 = from_raw565(0xec10);
pub const LIGHTCYAN: Rgb565 = from_raw565(0xdfff);
pub const LIGHTGOLDENRODYELLOW: Rgb565 = from_raw565(0xf7da);
pub const LIGHTGRAY: Rgb565 = from_raw565(0xd69a);
pub const LIGHTGREEN: Rgb565 = from_raw565(0x9772);
pub const LIGHTGREY: Rgb565 = from_raw565(0xd69a);
pub const LIGHTPINK: Rgb565 = from_raw565(0xfdb7);
pub const LIGHTSALMON: Rgb565 = from_raw565(0xfd0f);
pub const LIGHTSEAGREEN: Rgb565 = from_raw565(0x2595);
pub const LIGHTSKYBLUE: Rgb565 = from_raw565(0x867e);
pub const LIGHTSLATEGRAY: Rgb565 = from_raw565(0x7453);
pub const LIGHTSLATEGREY: Rgb565 = from_raw565(0x7453);
pub const LIGHTSTEELBLUE: Rgb565 = from_raw565(0xae1b);
pub const LIGHTYELLOW: Rgb565 = from_raw565(0xfffb);
pub const LIME: Rgb565 = from_raw565(0x07e0);
pub const LIMEGREEN: Rgb565 = from_raw565(0x3666);
pub const LINEN: Rgb565 = from_raw565(0xf77c);
pub const MAGENTA: Rgb565 = from_raw565(0xf81f);
pub const MAROON: Rgb565 = from_raw565(0x8000);
pub const MEDIUMAQUAMARINE: Rgb565 = from_raw565(0x6675);
pub const MEDIUMBLUE: Rgb565 = from_raw565(0x0019);
pub const MEDIUMORCHID: Rgb565 = from_raw565(0xbaba);
pub const MEDIUMPURPLE: Rgb565 = from_raw565(0x939b);
pub const MEDIUMSEAGREEN: Rgb565 = from_raw565(0x3d8e);
pub const MEDIUMSLATEBLUE: Rgb565 = from_raw565(0x7b5d);
pub const MEDIUMSPRINGGREEN: Rgb565 = from_raw565(0x07d3);
pub const MEDIUMTURQUOISE: Rgb565 = from_raw565(0x4e99);
pub const MEDIUMVIOLETRED: Rgb565 = from_raw565(0xc0b0);
pub const MIDNIGHTBLUE: Rgb565 = from_raw565(0x18ce);
pub const MINTCREAM: Rgb565 = from_raw565(0xf7fe);
pub const MISTYROSE: Rgb565 = from_raw565(0xff1b);
pub const MOCCASIN: Rgb565 = from_raw565(0xff16);
pub const NAVAJOWHITE: Rgb565 = from_raw565(0xfef5);
pub const NAVY: Rgb565 = from_raw565(0x0010);
pub const OLDLACE: Rgb565 = from_raw565(0xffbc);
pub const OLIVE: Rgb565 = from_raw565(0x8400);
pub const OLIVEDRAB: Rgb565 = from_raw565(0x6c64);
pub const ORANGE: Rgb565 = from_raw565(0xfd20);
pub const ORANGERED: Rgb565 = from_raw565(0xfa20);
pub const ORCHID: Rgb565 = from_raw565(0xdb9a);
pub const PALEGOLDENROD: Rgb565 = from_raw565(0xef35);
pub const PALEGREEN: Rgb565 = from_raw565(0x97d2);
pub const PALETURQUOISE: Rgb565 = from_raw565(0xaf7d);
pub const PALEVIOLETRED: Rgb565 = from_raw565(0xdb92);
pub const PAPAYAWHIP: Rgb565 = from_raw565(0xff7a);
pub const PEACHPUFF: Rgb565 = from_raw565(0xfed6);
pub const PERU: Rgb565 = from_raw565(0xcc28);
pub const PINK: Rgb565 = from_raw565(0xfdf9);
pub const PLUM: Rgb565 = from_raw565(0xdd1b);
pub const POWDERBLUE: Rgb565 = from_raw565(0xaefc);
pub const PURPLE: Rgb565 = from_raw565(0x8010);
pub const RED: Rgb565 = from_raw565(0xf800);
pub const ROSYBROWN: Rgb565 = from_raw565(0xbc71);
pub const ROYALBLUE: Rgb565 = from_raw565(0x435b);
pub const SADDLEBROWN: Rgb565 = from_raw565(0x8a22);
pub const SALMON: Rgb565 = from_raw565(0xf40e);
pub const SANDYBROWN: Rgb565 = from_raw565(0xf52c);
pub const SEAGREEN: Rgb565 = from_raw565(0x344b);
pub const SEASHELL: Rgb565 = from_raw565(0xffbd);
pub const SIENNA: Rgb565 = from_raw565(0x9a85);
pub const SILVER: Rgb565 = from_raw565(0xbdf7);
pub const SKYBLUE: Rgb565 = from_raw565(0x867d);
pub const SLATEBLUE: Rgb565 = from_raw565(0x6ad9);
pub const SLATEGRAY: Rgb565 = from_raw565(0x7412);
pub const SLATEGREY: Rgb565 = from_raw565(0x7412);
pub const SNOW: Rgb565 = from_raw565(0xffde);
pub const SPRINGGREEN: Rgb565 = from_raw565(0x07ef);
pub const STEELBLUE: Rgb565 = from_raw565(0x4c16);
pub const TAN: Rgb565 = from_raw565(0xd591);
pub const TEAL: Rgb565 = from_raw565(0x0410);
pub const THISTLE: Rgb565 = from_raw565(0xd5fa);
pub const TOMATO: Rgb565 = from_raw565(0xfb09);
pub const TURQUOISE: Rgb565 = from_raw565(0x46f9);
pub const VIOLET: Rgb565 = from_raw565(0xec1d);
pub const WHEAT: Rgb565 = from_raw565(0xf6f6);
pub const WHITE: Rgb565 = from_raw565(0xffff);
pub const WHITESMOKE: Rgb565 = from_raw565(0xf7be);
pub const YELLOW: Rgb565 = from_raw565(0xffe0);
pub const YELLOWGREEN: Rgb565 = from_raw565(0x9e66);

/// Named color lookup table for `#![no_std]` environments.
pub const CSS_COLOR_TABLE: [(&str, Rgb565); 147] = [
    ("aliceblue", ALICEBLUE),
    ("antiquewhite", ANTIQUEWHITE),
    ("aqua", AQUA),
    ("aquamarine", AQUAMARINE),
    ("azure", AZURE),
    ("beige", BEIGE),
    ("bisque", BISQUE),
    ("black", BLACK),
    ("blanchedalmond", BLANCHEDALMOND),
    ("blue", BLUE),
    ("blueviolet", BLUEVIOLET),
    ("brown", BROWN),
    ("burlywood", BURLYWOOD),
    ("cadetblue", CADETBLUE),
    ("chartreuse", CHARTREUSE),
    ("chocolate", CHOCOLATE),
    ("coral", CORAL),
    ("cornflowerblue", CORNFLOWERBLUE),
    ("cornsilk", CORNSILK),
    ("crimson", CRIMSON),
    ("cyan", CYAN),
    ("darkblue", DARKBLUE),
    ("darkcyan", DARKCYAN),
    ("darkgoldenrod", DARKGOLDENROD),
    ("darkgray", DARKGRAY),
    ("darkgreen", DARKGREEN),
    ("darkgrey", DARKGREY),
    ("darkkhaki", DARKKHAKI),
    ("darkmagenta", DARKMAGENTA),
    ("darkolivegreen", DARKOLIVEGREEN),
    ("darkorange", DARKORANGE),
    ("darkorchid", DARKORCHID),
    ("darkred", DARKRED),
    ("darksalmon", DARKSALMON),
    ("darkseagreen", DARKSEAGREEN),
    ("darkslateblue", DARKSLATEBLUE),
    ("darkslategray", DARKSLATEGRAY),
    ("darkslategrey", DARKSLATEGREY),
    ("darkturquoise", DARKTURQUOISE),
    ("darkviolet", DARKVIOLET),
    ("deeppink", DEEPPINK),
    ("deepskyblue", DEEPSKYBLUE),
    ("dimgray", DIMGRAY),
    ("dimgrey", DIMGREY),
    ("dodgerblue", DODGERBLUE),
    ("firebrick", FIREBRICK),
    ("floralwhite", FLORALWHITE),
    ("forestgreen", FORESTGREEN),
    ("fuchsia", FUCHSIA),
    ("gainsboro", GAINSBORO),
    ("ghostwhite", GHOSTWHITE),
    ("gold", GOLD),
    ("goldenrod", GOLDENROD),
    ("gray", GRAY),
    ("green", GREEN),
    ("greenyellow", GREENYELLOW),
    ("grey", GREY),
    ("honeydew", HONEYDEW),
    ("hotpink", HOTPINK),
    ("indianred", INDIANRED),
    ("indigo", INDIGO),
    ("ivory", IVORY),
    ("khaki", KHAKI),
    ("lavender", LAVENDER),
    ("lavenderblush", LAVENDERBLUSH),
    ("lawngreen", LAWNGREEN),
    ("lemonchiffon", LEMONCHIFFON),
    ("lightblue", LIGHTBLUE),
    ("lightcoral", LIGHTCORAL),
    ("lightcyan", LIGHTCYAN),
    ("lightgoldenrodyellow", LIGHTGOLDENRODYELLOW),
    ("lightgray", LIGHTGRAY),
    ("lightgreen", LIGHTGREEN),
    ("lightgrey", LIGHTGREY),
    ("lightpink", LIGHTPINK),
    ("lightsalmon", LIGHTSALMON),
    ("lightseagreen", LIGHTSEAGREEN),
    ("lightskyblue", LIGHTSKYBLUE),
    ("lightslategray", LIGHTSLATEGRAY),
    ("lightslategrey", LIGHTSLATEGREY),
    ("lightsteelblue", LIGHTSTEELBLUE),
    ("lightyellow", LIGHTYELLOW),
    ("lime", LIME),
    ("limegreen", LIMEGREEN),
    ("linen", LINEN),
    ("magenta", MAGENTA),
    ("maroon", MAROON),
    ("mediumaquamarine", MEDIUMAQUAMARINE),
    ("mediumblue", MEDIUMBLUE),
    ("mediumorchid", MEDIUMORCHID),
    ("mediumpurple", MEDIUMPURPLE),
    ("mediumseagreen", MEDIUMSEAGREEN),
    ("mediumslateblue", MEDIUMSLATEBLUE),
    ("mediumspringgreen", MEDIUMSPRINGGREEN),
    ("mediumturquoise", MEDIUMTURQUOISE),
    ("mediumvioletred", MEDIUMVIOLETRED),
    ("midnightblue", MIDNIGHTBLUE),
    ("mintcream", MINTCREAM),
    ("mistyrose", MISTYROSE),
    ("moccasin", MOCCASIN),
    ("navajowhite", NAVAJOWHITE),
    ("navy", NAVY),
    ("oldlace", OLDLACE),
    ("olive", OLIVE),
    ("olivedrab", OLIVEDRAB),
    ("orange", ORANGE),
    ("orangered", ORANGERED),
    ("orchid", ORCHID),
    ("palegoldenrod", PALEGOLDENROD),
    ("palegreen", PALEGREEN),
    ("paleturquoise", PALETURQUOISE),
    ("palevioletred", PALEVIOLETRED),
    ("papayawhip", PAPAYAWHIP),
    ("peachpuff", PEACHPUFF),
    ("peru", PERU),
    ("pink", PINK),
    ("plum", PLUM),
    ("powderblue", POWDERBLUE),
    ("purple", PURPLE),
    ("red", RED),
    ("rosybrown", ROSYBROWN),
    ("royalblue", ROYALBLUE),
    ("saddlebrown", SADDLEBROWN),
    ("salmon", SALMON),
    ("sandybrown", SANDYBROWN),
    ("seagreen", SEAGREEN),
    ("seashell", SEASHELL),
    ("sienna", SIENNA),
    ("silver", SILVER),
    ("skyblue", SKYBLUE),
    ("slateblue", SLATEBLUE),
    ("slategray", SLATEGRAY),
    ("slategrey", SLATEGREY),
    ("snow", SNOW),
    ("springgreen", SPRINGGREEN),
    ("steelblue", STEELBLUE),
    ("tan", TAN),
    ("teal", TEAL),
    ("thistle", THISTLE),
    ("tomato", TOMATO),
    ("turquoise", TURQUOISE),
    ("violet", VIOLET),
    ("wheat", WHEAT),
    ("white", WHITE),
    ("whitesmoke", WHITESMOKE),
    ("yellow", YELLOW),
    ("yellowgreen", YELLOWGREEN),
];

/// Lookup a standard CSS color by name (case-insensitive, zero heap allocation).
pub fn from_name(name: &str) -> Option<Rgb565> {
    let mut i = 0;
    while i < CSS_COLOR_TABLE.len() {
        if CSS_COLOR_TABLE[i].0.eq_ignore_ascii_case(name) {
            return Some(CSS_COLOR_TABLE[i].1);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_colors_lookup() {
        assert_eq!(from_name("aliceblue"), Some(ALICEBLUE));
        assert_eq!(from_name("AliceBlue"), Some(ALICEBLUE));
        assert_eq!(from_name("forestgreen"), Some(FORESTGREEN));
        assert_eq!(from_name("crimson"), Some(CRIMSON));
        assert_eq!(from_name("notacolor"), None);
    }
}
