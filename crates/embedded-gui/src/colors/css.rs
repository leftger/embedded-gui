//! Standard CSS / SVG Level 4 color palette ported from `bevy_color::palettes::css`.
//!
//! Contains all 148 standard CSS named colors as packed `const Rgb565` values for `#![no_std]` MCU targets.

use embedded_graphics_core::pixelcolor::Rgb565;

/// Standard CSS color `ALICE_BLUE` (RGB565 `0xEFBF`).
pub const ALICE_BLUE: Rgb565 = crate::colors::from_raw565(0xefbf);
/// Standard CSS color `ANTIQUE_WHITE` (RGB565 `0xF75A`).
pub const ANTIQUE_WHITE: Rgb565 = crate::colors::from_raw565(0xf75a);
/// Standard CSS color `AQUA` (RGB565 `0x07FF`).
pub const AQUA: Rgb565 = crate::colors::from_raw565(0x07ff);
/// Standard CSS color `AQUAMARINE` (RGB565 `0x7FFA`).
pub const AQUAMARINE: Rgb565 = crate::colors::from_raw565(0x7ffa);
/// Standard CSS color `AZURE` (RGB565 `0xEFFF`).
pub const AZURE: Rgb565 = crate::colors::from_raw565(0xefff);
/// Standard CSS color `BEIGE` (RGB565 `0xF7BB`).
pub const BEIGE: Rgb565 = crate::colors::from_raw565(0xf7bb);
/// Standard CSS color `BISQUE` (RGB565 `0xFF18`).
pub const BISQUE: Rgb565 = crate::colors::from_raw565(0xff18);
/// Standard CSS color `BLACK` (RGB565 `0x0000`).
pub const BLACK: Rgb565 = crate::colors::from_raw565(0x0000);
/// Standard CSS color `BLANCHED_ALMOND` (RGB565 `0xFF59`).
pub const BLANCHED_ALMOND: Rgb565 = crate::colors::from_raw565(0xff59);
/// Standard CSS color `BLUE` (RGB565 `0x001F`).
pub const BLUE: Rgb565 = crate::colors::from_raw565(0x001f);
/// Standard CSS color `BLUE_VIOLET` (RGB565 `0x897B`).
pub const BLUE_VIOLET: Rgb565 = crate::colors::from_raw565(0x897b);
/// Standard CSS color `BROWN` (RGB565 `0xA145`).
pub const BROWN: Rgb565 = crate::colors::from_raw565(0xa145);
/// Standard CSS color `BURLYWOOD` (RGB565 `0xDDB0`).
pub const BURLYWOOD: Rgb565 = crate::colors::from_raw565(0xddb0);
/// Standard CSS color `CADET_BLUE` (RGB565 `0x64F3`).
pub const CADET_BLUE: Rgb565 = crate::colors::from_raw565(0x64f3);
/// Standard CSS color `CHARTREUSE` (RGB565 `0x7FE0`).
pub const CHARTREUSE: Rgb565 = crate::colors::from_raw565(0x7fe0);
/// Standard CSS color `CHOCOLATE` (RGB565 `0xD344`).
pub const CHOCOLATE: Rgb565 = crate::colors::from_raw565(0xd344);
/// Standard CSS color `CORAL` (RGB565 `0xFBEA`).
pub const CORAL: Rgb565 = crate::colors::from_raw565(0xfbea);
/// Standard CSS color `CORNFLOWER_BLUE` (RGB565 `0x64BD`).
pub const CORNFLOWER_BLUE: Rgb565 = crate::colors::from_raw565(0x64bd);
/// Standard CSS color `CORNSILK` (RGB565 `0xFFBB`).
pub const CORNSILK: Rgb565 = crate::colors::from_raw565(0xffbb);
/// Standard CSS color `CRIMSON` (RGB565 `0xD8A7`).
pub const CRIMSON: Rgb565 = crate::colors::from_raw565(0xd8a7);
/// Standard CSS color `DARK_BLUE` (RGB565 `0x0011`).
pub const DARK_BLUE: Rgb565 = crate::colors::from_raw565(0x0011);
/// Standard CSS color `DARK_CYAN` (RGB565 `0x0451`).
pub const DARK_CYAN: Rgb565 = crate::colors::from_raw565(0x0451);
/// Standard CSS color `DARK_GOLDENROD` (RGB565 `0xB421`).
pub const DARK_GOLDENROD: Rgb565 = crate::colors::from_raw565(0xb421);
/// Standard CSS color `DARK_GRAY` (RGB565 `0xAD55`).
pub const DARK_GRAY: Rgb565 = crate::colors::from_raw565(0xad55);
/// Standard CSS color `DARK_GREEN` (RGB565 `0x0320`).
pub const DARK_GREEN: Rgb565 = crate::colors::from_raw565(0x0320);
/// Standard CSS color `DARK_GREY` (RGB565 `0xAD55`).
pub const DARK_GREY: Rgb565 = crate::colors::from_raw565(0xad55);
/// Standard CSS color `DARK_KHAKI` (RGB565 `0xBDAD`).
pub const DARK_KHAKI: Rgb565 = crate::colors::from_raw565(0xbdad);
/// Standard CSS color `DARK_MAGENTA` (RGB565 `0x8811`).
pub const DARK_MAGENTA: Rgb565 = crate::colors::from_raw565(0x8811);
/// Standard CSS color `DARK_OLIVEGREEN` (RGB565 `0x5346`).
pub const DARK_OLIVEGREEN: Rgb565 = crate::colors::from_raw565(0x5346);
/// Standard CSS color `DARK_ORANGE` (RGB565 `0xFC60`).
pub const DARK_ORANGE: Rgb565 = crate::colors::from_raw565(0xfc60);
/// Standard CSS color `DARK_ORCHID` (RGB565 `0x9999`).
pub const DARK_ORCHID: Rgb565 = crate::colors::from_raw565(0x9999);
/// Standard CSS color `DARK_RED` (RGB565 `0x8800`).
pub const DARK_RED: Rgb565 = crate::colors::from_raw565(0x8800);
/// Standard CSS color `DARK_SALMON` (RGB565 `0xE4AF`).
pub const DARK_SALMON: Rgb565 = crate::colors::from_raw565(0xe4af);
/// Standard CSS color `DARK_SEA_GREEN` (RGB565 `0x8DD1`).
pub const DARK_SEA_GREEN: Rgb565 = crate::colors::from_raw565(0x8dd1);
/// Standard CSS color `DARK_SLATE_BLUE` (RGB565 `0x49F1`).
pub const DARK_SLATE_BLUE: Rgb565 = crate::colors::from_raw565(0x49f1);
/// Standard CSS color `DARK_SLATE_GRAY` (RGB565 `0x328A`).
pub const DARK_SLATE_GRAY: Rgb565 = crate::colors::from_raw565(0x328a);
/// Standard CSS color `DARK_SLATE_GREY` (RGB565 `0x328A`).
pub const DARK_SLATE_GREY: Rgb565 = crate::colors::from_raw565(0x328a);
/// Standard CSS color `DARK_TURQUOISE` (RGB565 `0x0679`).
pub const DARK_TURQUOISE: Rgb565 = crate::colors::from_raw565(0x0679);
/// Standard CSS color `DARK_VIOLET` (RGB565 `0x901A`).
pub const DARK_VIOLET: Rgb565 = crate::colors::from_raw565(0x901a);
/// Standard CSS color `DEEP_PINK` (RGB565 `0xF8B2`).
pub const DEEP_PINK: Rgb565 = crate::colors::from_raw565(0xf8b2);
/// Standard CSS color `DEEP_SKY_BLUE` (RGB565 `0x05FF`).
pub const DEEP_SKY_BLUE: Rgb565 = crate::colors::from_raw565(0x05ff);
/// Standard CSS color `DIM_GRAY` (RGB565 `0x6B4D`).
pub const DIM_GRAY: Rgb565 = crate::colors::from_raw565(0x6b4d);
/// Standard CSS color `DIM_GREY` (RGB565 `0x6B4D`).
pub const DIM_GREY: Rgb565 = crate::colors::from_raw565(0x6b4d);
/// Standard CSS color `DODGER_BLUE` (RGB565 `0x249F`).
pub const DODGER_BLUE: Rgb565 = crate::colors::from_raw565(0x249f);
/// Standard CSS color `FIRE_BRICK` (RGB565 `0xB104`).
pub const FIRE_BRICK: Rgb565 = crate::colors::from_raw565(0xb104);
/// Standard CSS color `FLORAL_WHITE` (RGB565 `0xFFDD`).
pub const FLORAL_WHITE: Rgb565 = crate::colors::from_raw565(0xffdd);
/// Standard CSS color `FOREST_GREEN` (RGB565 `0x2444`).
pub const FOREST_GREEN: Rgb565 = crate::colors::from_raw565(0x2444);
/// Standard CSS color `FUCHSIA` (RGB565 `0xF81F`).
pub const FUCHSIA: Rgb565 = crate::colors::from_raw565(0xf81f);
/// Standard CSS color `GAINSBORO` (RGB565 `0xDEDB`).
pub const GAINSBORO: Rgb565 = crate::colors::from_raw565(0xdedb);
/// Standard CSS color `GHOST_WHITE` (RGB565 `0xF7BF`).
pub const GHOST_WHITE: Rgb565 = crate::colors::from_raw565(0xf7bf);
/// Standard CSS color `GOLD` (RGB565 `0xFEA0`).
pub const GOLD: Rgb565 = crate::colors::from_raw565(0xfea0);
/// Standard CSS color `GOLDENROD` (RGB565 `0xDD24`).
pub const GOLDENROD: Rgb565 = crate::colors::from_raw565(0xdd24);
/// Standard CSS color `GRAY` (RGB565 `0x8410`).
pub const GRAY: Rgb565 = crate::colors::from_raw565(0x8410);
/// Standard CSS color `GREEN` (RGB565 `0x0400`).
pub const GREEN: Rgb565 = crate::colors::from_raw565(0x0400);
/// Standard CSS color `GREEN_YELLOW` (RGB565 `0xAFE6`).
pub const GREEN_YELLOW: Rgb565 = crate::colors::from_raw565(0xafe6);
/// Standard CSS color `GREY` (RGB565 `0x8410`).
pub const GREY: Rgb565 = crate::colors::from_raw565(0x8410);
/// Standard CSS color `HONEYDEW` (RGB565 `0xEFFD`).
pub const HONEYDEW: Rgb565 = crate::colors::from_raw565(0xeffd);
/// Standard CSS color `HOT_PINK` (RGB565 `0xFB56`).
pub const HOT_PINK: Rgb565 = crate::colors::from_raw565(0xfb56);
/// Standard CSS color `INDIAN_RED` (RGB565 `0xCAEB`).
pub const INDIAN_RED: Rgb565 = crate::colors::from_raw565(0xcaeb);
/// Standard CSS color `INDIGO` (RGB565 `0x4810`).
pub const INDIGO: Rgb565 = crate::colors::from_raw565(0x4810);
/// Standard CSS color `IVORY` (RGB565 `0xFFFD`).
pub const IVORY: Rgb565 = crate::colors::from_raw565(0xfffd);
/// Standard CSS color `KHAKI` (RGB565 `0xEF31`).
pub const KHAKI: Rgb565 = crate::colors::from_raw565(0xef31);
/// Standard CSS color `LAVENDER` (RGB565 `0xE73E`).
pub const LAVENDER: Rgb565 = crate::colors::from_raw565(0xe73e);
/// Standard CSS color `LAVENDER_BLUSH` (RGB565 `0xFF7E`).
pub const LAVENDER_BLUSH: Rgb565 = crate::colors::from_raw565(0xff7e);
/// Standard CSS color `LAWN_GREEN` (RGB565 `0x7FC0`).
pub const LAWN_GREEN: Rgb565 = crate::colors::from_raw565(0x7fc0);
/// Standard CSS color `LEMON_CHIFFON` (RGB565 `0xFFD9`).
pub const LEMON_CHIFFON: Rgb565 = crate::colors::from_raw565(0xffd9);
/// Standard CSS color `LIGHT_BLUE` (RGB565 `0xAEBC`).
pub const LIGHT_BLUE: Rgb565 = crate::colors::from_raw565(0xaebc);
/// Standard CSS color `LIGHT_CORAL` (RGB565 `0xEC10`).
pub const LIGHT_CORAL: Rgb565 = crate::colors::from_raw565(0xec10);
/// Standard CSS color `LIGHT_CYAN` (RGB565 `0xDFFF`).
pub const LIGHT_CYAN: Rgb565 = crate::colors::from_raw565(0xdfff);
/// Standard CSS color `LIGHT_GOLDENROD_YELLOW` (RGB565 `0xF7DA`).
pub const LIGHT_GOLDENROD_YELLOW: Rgb565 = crate::colors::from_raw565(0xf7da);
/// Standard CSS color `LIGHT_GRAY` (RGB565 `0xD69A`).
pub const LIGHT_GRAY: Rgb565 = crate::colors::from_raw565(0xd69a);
/// Standard CSS color `LIGHT_GREEN` (RGB565 `0x9772`).
pub const LIGHT_GREEN: Rgb565 = crate::colors::from_raw565(0x9772);
/// Standard CSS color `LIGHT_GREY` (RGB565 `0xD69A`).
pub const LIGHT_GREY: Rgb565 = crate::colors::from_raw565(0xd69a);
/// Standard CSS color `LIGHT_PINK` (RGB565 `0xFDB7`).
pub const LIGHT_PINK: Rgb565 = crate::colors::from_raw565(0xfdb7);
/// Standard CSS color `LIGHT_SALMON` (RGB565 `0xFD0F`).
pub const LIGHT_SALMON: Rgb565 = crate::colors::from_raw565(0xfd0f);
/// Standard CSS color `LIGHT_SEA_GREEN` (RGB565 `0x2595`).
pub const LIGHT_SEA_GREEN: Rgb565 = crate::colors::from_raw565(0x2595);
/// Standard CSS color `LIGHT_SKY_BLUE` (RGB565 `0x867E`).
pub const LIGHT_SKY_BLUE: Rgb565 = crate::colors::from_raw565(0x867e);
/// Standard CSS color `LIGHT_SLATE_GRAY` (RGB565 `0x7453`).
pub const LIGHT_SLATE_GRAY: Rgb565 = crate::colors::from_raw565(0x7453);
/// Standard CSS color `LIGHT_SLATE_GREY` (RGB565 `0x7453`).
pub const LIGHT_SLATE_GREY: Rgb565 = crate::colors::from_raw565(0x7453);
/// Standard CSS color `LIGHT_STEEL_BLUE` (RGB565 `0xAE1B`).
pub const LIGHT_STEEL_BLUE: Rgb565 = crate::colors::from_raw565(0xae1b);
/// Standard CSS color `LIGHT_YELLOW` (RGB565 `0xFFFB`).
pub const LIGHT_YELLOW: Rgb565 = crate::colors::from_raw565(0xfffb);
/// Standard CSS color `LIME` (RGB565 `0x07E0`).
pub const LIME: Rgb565 = crate::colors::from_raw565(0x07e0);
/// Standard CSS color `LIMEGREEN` (RGB565 `0x3666`).
pub const LIMEGREEN: Rgb565 = crate::colors::from_raw565(0x3666);
/// Standard CSS color `LINEN` (RGB565 `0xF77C`).
pub const LINEN: Rgb565 = crate::colors::from_raw565(0xf77c);
/// Standard CSS color `MAGENTA` (RGB565 `0xF81F`).
pub const MAGENTA: Rgb565 = crate::colors::from_raw565(0xf81f);
/// Standard CSS color `MAROON` (RGB565 `0x8000`).
pub const MAROON: Rgb565 = crate::colors::from_raw565(0x8000);
/// Standard CSS color `MEDIUM_AQUAMARINE` (RGB565 `0x6675`).
pub const MEDIUM_AQUAMARINE: Rgb565 = crate::colors::from_raw565(0x6675);
/// Standard CSS color `MEDIUM_BLUE` (RGB565 `0x0019`).
pub const MEDIUM_BLUE: Rgb565 = crate::colors::from_raw565(0x0019);
/// Standard CSS color `MEDIUM_ORCHID` (RGB565 `0xBABA`).
pub const MEDIUM_ORCHID: Rgb565 = crate::colors::from_raw565(0xbaba);
/// Standard CSS color `MEDIUM_PURPLE` (RGB565 `0x939B`).
pub const MEDIUM_PURPLE: Rgb565 = crate::colors::from_raw565(0x939b);
/// Standard CSS color `MEDIUM_SEA_GREEN` (RGB565 `0x3D8E`).
pub const MEDIUM_SEA_GREEN: Rgb565 = crate::colors::from_raw565(0x3d8e);
/// Standard CSS color `MEDIUM_SLATE_BLUE` (RGB565 `0x7B5D`).
pub const MEDIUM_SLATE_BLUE: Rgb565 = crate::colors::from_raw565(0x7b5d);
/// Standard CSS color `MEDIUM_SPRING_GREEN` (RGB565 `0x07D3`).
pub const MEDIUM_SPRING_GREEN: Rgb565 = crate::colors::from_raw565(0x07d3);
/// Standard CSS color `MEDIUM_TURQUOISE` (RGB565 `0x4E99`).
pub const MEDIUM_TURQUOISE: Rgb565 = crate::colors::from_raw565(0x4e99);
/// Standard CSS color `MEDIUM_VIOLET_RED` (RGB565 `0xC0B0`).
pub const MEDIUM_VIOLET_RED: Rgb565 = crate::colors::from_raw565(0xc0b0);
/// Standard CSS color `MIDNIGHT_BLUE` (RGB565 `0x18CE`).
pub const MIDNIGHT_BLUE: Rgb565 = crate::colors::from_raw565(0x18ce);
/// Standard CSS color `MINT_CREAM` (RGB565 `0xF7FE`).
pub const MINT_CREAM: Rgb565 = crate::colors::from_raw565(0xf7fe);
/// Standard CSS color `MISTY_ROSE` (RGB565 `0xFF1B`).
pub const MISTY_ROSE: Rgb565 = crate::colors::from_raw565(0xff1b);
/// Standard CSS color `MOCCASIN` (RGB565 `0xFF16`).
pub const MOCCASIN: Rgb565 = crate::colors::from_raw565(0xff16);
/// Standard CSS color `NAVAJO_WHITE` (RGB565 `0xFEF5`).
pub const NAVAJO_WHITE: Rgb565 = crate::colors::from_raw565(0xfef5);
/// Standard CSS color `NAVY` (RGB565 `0x0010`).
pub const NAVY: Rgb565 = crate::colors::from_raw565(0x0010);
/// Standard CSS color `OLD_LACE` (RGB565 `0xFFBC`).
pub const OLD_LACE: Rgb565 = crate::colors::from_raw565(0xffbc);
/// Standard CSS color `OLIVE` (RGB565 `0x8400`).
pub const OLIVE: Rgb565 = crate::colors::from_raw565(0x8400);
/// Standard CSS color `OLIVE_DRAB` (RGB565 `0x6C64`).
pub const OLIVE_DRAB: Rgb565 = crate::colors::from_raw565(0x6c64);
/// Standard CSS color `ORANGE` (RGB565 `0xFD20`).
pub const ORANGE: Rgb565 = crate::colors::from_raw565(0xfd20);
/// Standard CSS color `ORANGE_RED` (RGB565 `0xFA20`).
pub const ORANGE_RED: Rgb565 = crate::colors::from_raw565(0xfa20);
/// Standard CSS color `ORCHID` (RGB565 `0xDB9A`).
pub const ORCHID: Rgb565 = crate::colors::from_raw565(0xdb9a);
/// Standard CSS color `PALE_GOLDENROD` (RGB565 `0xEF35`).
pub const PALE_GOLDENROD: Rgb565 = crate::colors::from_raw565(0xef35);
/// Standard CSS color `PALE_GREEN` (RGB565 `0x97D2`).
pub const PALE_GREEN: Rgb565 = crate::colors::from_raw565(0x97d2);
/// Standard CSS color `PALE_TURQUOISE` (RGB565 `0xAF7D`).
pub const PALE_TURQUOISE: Rgb565 = crate::colors::from_raw565(0xaf7d);
/// Standard CSS color `PALE_VIOLETRED` (RGB565 `0xDB92`).
pub const PALE_VIOLETRED: Rgb565 = crate::colors::from_raw565(0xdb92);
/// Standard CSS color `PAPAYA_WHIP` (RGB565 `0xFF7A`).
pub const PAPAYA_WHIP: Rgb565 = crate::colors::from_raw565(0xff7a);
/// Standard CSS color `PEACHPUFF` (RGB565 `0xFED6`).
pub const PEACHPUFF: Rgb565 = crate::colors::from_raw565(0xfed6);
/// Standard CSS color `PERU` (RGB565 `0xCC28`).
pub const PERU: Rgb565 = crate::colors::from_raw565(0xcc28);
/// Standard CSS color `PINK` (RGB565 `0xFDF9`).
pub const PINK: Rgb565 = crate::colors::from_raw565(0xfdf9);
/// Standard CSS color `PLUM` (RGB565 `0xDD1B`).
pub const PLUM: Rgb565 = crate::colors::from_raw565(0xdd1b);
/// Standard CSS color `POWDER_BLUE` (RGB565 `0xAEFC`).
pub const POWDER_BLUE: Rgb565 = crate::colors::from_raw565(0xaefc);
/// Standard CSS color `PURPLE` (RGB565 `0x8010`).
pub const PURPLE: Rgb565 = crate::colors::from_raw565(0x8010);
/// Standard CSS color `REBECCA_PURPLE` (RGB565 `0x61B3`).
pub const REBECCA_PURPLE: Rgb565 = crate::colors::from_raw565(0x61b3);
/// Standard CSS color `RED` (RGB565 `0xF800`).
pub const RED: Rgb565 = crate::colors::from_raw565(0xf800);
/// Standard CSS color `ROSY_BROWN` (RGB565 `0xBC71`).
pub const ROSY_BROWN: Rgb565 = crate::colors::from_raw565(0xbc71);
/// Standard CSS color `ROYAL_BLUE` (RGB565 `0x435B`).
pub const ROYAL_BLUE: Rgb565 = crate::colors::from_raw565(0x435b);
/// Standard CSS color `SADDLE_BROWN` (RGB565 `0x8A22`).
pub const SADDLE_BROWN: Rgb565 = crate::colors::from_raw565(0x8a22);
/// Standard CSS color `SALMON` (RGB565 `0xF40E`).
pub const SALMON: Rgb565 = crate::colors::from_raw565(0xf40e);
/// Standard CSS color `SANDY_BROWN` (RGB565 `0xF52C`).
pub const SANDY_BROWN: Rgb565 = crate::colors::from_raw565(0xf52c);
/// Standard CSS color `SEASHELL` (RGB565 `0xFFBD`).
pub const SEASHELL: Rgb565 = crate::colors::from_raw565(0xffbd);
/// Standard CSS color `SEA_GREEN` (RGB565 `0x344B`).
pub const SEA_GREEN: Rgb565 = crate::colors::from_raw565(0x344b);
/// Standard CSS color `SIENNA` (RGB565 `0x9A85`).
pub const SIENNA: Rgb565 = crate::colors::from_raw565(0x9a85);
/// Standard CSS color `SILVER` (RGB565 `0xBDF7`).
pub const SILVER: Rgb565 = crate::colors::from_raw565(0xbdf7);
/// Standard CSS color `SKY_BLUE` (RGB565 `0x867D`).
pub const SKY_BLUE: Rgb565 = crate::colors::from_raw565(0x867d);
/// Standard CSS color `SLATE_BLUE` (RGB565 `0x6AD9`).
pub const SLATE_BLUE: Rgb565 = crate::colors::from_raw565(0x6ad9);
/// Standard CSS color `SLATE_GRAY` (RGB565 `0x7412`).
pub const SLATE_GRAY: Rgb565 = crate::colors::from_raw565(0x7412);
/// Standard CSS color `SLATE_GREY` (RGB565 `0x7412`).
pub const SLATE_GREY: Rgb565 = crate::colors::from_raw565(0x7412);
/// Standard CSS color `SNOW` (RGB565 `0xFFDE`).
pub const SNOW: Rgb565 = crate::colors::from_raw565(0xffde);
/// Standard CSS color `SPRING_GREEN` (RGB565 `0x07EF`).
pub const SPRING_GREEN: Rgb565 = crate::colors::from_raw565(0x07ef);
/// Standard CSS color `STEEL_BLUE` (RGB565 `0x4C16`).
pub const STEEL_BLUE: Rgb565 = crate::colors::from_raw565(0x4c16);
/// Standard CSS color `TAN` (RGB565 `0xD591`).
pub const TAN: Rgb565 = crate::colors::from_raw565(0xd591);
/// Standard CSS color `TEAL` (RGB565 `0x0410`).
pub const TEAL: Rgb565 = crate::colors::from_raw565(0x0410);
/// Standard CSS color `THISTLE` (RGB565 `0xD5FA`).
pub const THISTLE: Rgb565 = crate::colors::from_raw565(0xd5fa);
/// Standard CSS color `TOMATO` (RGB565 `0xFB09`).
pub const TOMATO: Rgb565 = crate::colors::from_raw565(0xfb09);
/// Standard CSS color `TURQUOISE` (RGB565 `0x46F9`).
pub const TURQUOISE: Rgb565 = crate::colors::from_raw565(0x46f9);
/// Standard CSS color `VIOLET` (RGB565 `0xEC1D`).
pub const VIOLET: Rgb565 = crate::colors::from_raw565(0xec1d);
/// Standard CSS color `WHEAT` (RGB565 `0xF6F6`).
pub const WHEAT: Rgb565 = crate::colors::from_raw565(0xf6f6);
/// Standard CSS color `WHITE` (RGB565 `0xFFFF`).
pub const WHITE: Rgb565 = crate::colors::from_raw565(0xffff);
/// Standard CSS color `WHITE_SMOKE` (RGB565 `0xF7BE`).
pub const WHITE_SMOKE: Rgb565 = crate::colors::from_raw565(0xf7be);
/// Standard CSS color `YELLOW` (RGB565 `0xFFE0`).
pub const YELLOW: Rgb565 = crate::colors::from_raw565(0xffe0);
/// Standard CSS color `YELLOW_GREEN` (RGB565 `0x9E66`).
pub const YELLOW_GREEN: Rgb565 = crate::colors::from_raw565(0x9e66);

/// Lookup table of standard CSS colors for `#![no_std]` runtime name resolution.
pub const CSS_PALETTE_TABLE: [(&str, Rgb565); 147] = [
    ("alice-blue", ALICE_BLUE),
    ("antique-white", ANTIQUE_WHITE),
    ("aqua", AQUA),
    ("aquamarine", AQUAMARINE),
    ("azure", AZURE),
    ("beige", BEIGE),
    ("bisque", BISQUE),
    ("black", BLACK),
    ("blanched-almond", BLANCHED_ALMOND),
    ("blue", BLUE),
    ("blue-violet", BLUE_VIOLET),
    ("brown", BROWN),
    ("burlywood", BURLYWOOD),
    ("cadet-blue", CADET_BLUE),
    ("chartreuse", CHARTREUSE),
    ("chocolate", CHOCOLATE),
    ("coral", CORAL),
    ("cornflower-blue", CORNFLOWER_BLUE),
    ("cornsilk", CORNSILK),
    ("crimson", CRIMSON),
    ("dark-blue", DARK_BLUE),
    ("dark-cyan", DARK_CYAN),
    ("dark-goldenrod", DARK_GOLDENROD),
    ("dark-gray", DARK_GRAY),
    ("dark-green", DARK_GREEN),
    ("dark-grey", DARK_GREY),
    ("dark-khaki", DARK_KHAKI),
    ("dark-magenta", DARK_MAGENTA),
    ("dark-olivegreen", DARK_OLIVEGREEN),
    ("dark-orange", DARK_ORANGE),
    ("dark-orchid", DARK_ORCHID),
    ("dark-red", DARK_RED),
    ("dark-salmon", DARK_SALMON),
    ("dark-sea-green", DARK_SEA_GREEN),
    ("dark-slate-blue", DARK_SLATE_BLUE),
    ("dark-slate-gray", DARK_SLATE_GRAY),
    ("dark-slate-grey", DARK_SLATE_GREY),
    ("dark-turquoise", DARK_TURQUOISE),
    ("dark-violet", DARK_VIOLET),
    ("deep-pink", DEEP_PINK),
    ("deep-sky-blue", DEEP_SKY_BLUE),
    ("dim-gray", DIM_GRAY),
    ("dim-grey", DIM_GREY),
    ("dodger-blue", DODGER_BLUE),
    ("fire-brick", FIRE_BRICK),
    ("floral-white", FLORAL_WHITE),
    ("forest-green", FOREST_GREEN),
    ("fuchsia", FUCHSIA),
    ("gainsboro", GAINSBORO),
    ("ghost-white", GHOST_WHITE),
    ("gold", GOLD),
    ("goldenrod", GOLDENROD),
    ("gray", GRAY),
    ("green", GREEN),
    ("green-yellow", GREEN_YELLOW),
    ("grey", GREY),
    ("honeydew", HONEYDEW),
    ("hot-pink", HOT_PINK),
    ("indian-red", INDIAN_RED),
    ("indigo", INDIGO),
    ("ivory", IVORY),
    ("khaki", KHAKI),
    ("lavender", LAVENDER),
    ("lavender-blush", LAVENDER_BLUSH),
    ("lawn-green", LAWN_GREEN),
    ("lemon-chiffon", LEMON_CHIFFON),
    ("light-blue", LIGHT_BLUE),
    ("light-coral", LIGHT_CORAL),
    ("light-cyan", LIGHT_CYAN),
    ("light-goldenrod-yellow", LIGHT_GOLDENROD_YELLOW),
    ("light-gray", LIGHT_GRAY),
    ("light-green", LIGHT_GREEN),
    ("light-grey", LIGHT_GREY),
    ("light-pink", LIGHT_PINK),
    ("light-salmon", LIGHT_SALMON),
    ("light-sea-green", LIGHT_SEA_GREEN),
    ("light-sky-blue", LIGHT_SKY_BLUE),
    ("light-slate-gray", LIGHT_SLATE_GRAY),
    ("light-slate-grey", LIGHT_SLATE_GREY),
    ("light-steel-blue", LIGHT_STEEL_BLUE),
    ("light-yellow", LIGHT_YELLOW),
    ("lime", LIME),
    ("limegreen", LIMEGREEN),
    ("linen", LINEN),
    ("magenta", MAGENTA),
    ("maroon", MAROON),
    ("medium-aquamarine", MEDIUM_AQUAMARINE),
    ("medium-blue", MEDIUM_BLUE),
    ("medium-orchid", MEDIUM_ORCHID),
    ("medium-purple", MEDIUM_PURPLE),
    ("medium-sea-green", MEDIUM_SEA_GREEN),
    ("medium-slate-blue", MEDIUM_SLATE_BLUE),
    ("medium-spring-green", MEDIUM_SPRING_GREEN),
    ("medium-turquoise", MEDIUM_TURQUOISE),
    ("medium-violet-red", MEDIUM_VIOLET_RED),
    ("midnight-blue", MIDNIGHT_BLUE),
    ("mint-cream", MINT_CREAM),
    ("misty-rose", MISTY_ROSE),
    ("moccasin", MOCCASIN),
    ("navajo-white", NAVAJO_WHITE),
    ("navy", NAVY),
    ("old-lace", OLD_LACE),
    ("olive", OLIVE),
    ("olive-drab", OLIVE_DRAB),
    ("orange", ORANGE),
    ("orange-red", ORANGE_RED),
    ("orchid", ORCHID),
    ("pale-goldenrod", PALE_GOLDENROD),
    ("pale-green", PALE_GREEN),
    ("pale-turquoise", PALE_TURQUOISE),
    ("pale-violetred", PALE_VIOLETRED),
    ("papaya-whip", PAPAYA_WHIP),
    ("peachpuff", PEACHPUFF),
    ("peru", PERU),
    ("pink", PINK),
    ("plum", PLUM),
    ("powder-blue", POWDER_BLUE),
    ("purple", PURPLE),
    ("rebecca-purple", REBECCA_PURPLE),
    ("red", RED),
    ("rosy-brown", ROSY_BROWN),
    ("royal-blue", ROYAL_BLUE),
    ("saddle-brown", SADDLE_BROWN),
    ("salmon", SALMON),
    ("sandy-brown", SANDY_BROWN),
    ("seashell", SEASHELL),
    ("sea-green", SEA_GREEN),
    ("sienna", SIENNA),
    ("silver", SILVER),
    ("sky-blue", SKY_BLUE),
    ("slate-blue", SLATE_BLUE),
    ("slate-gray", SLATE_GRAY),
    ("slate-grey", SLATE_GREY),
    ("snow", SNOW),
    ("spring-green", SPRING_GREEN),
    ("steel-blue", STEEL_BLUE),
    ("tan", TAN),
    ("teal", TEAL),
    ("thistle", THISTLE),
    ("tomato", TOMATO),
    ("turquoise", TURQUOISE),
    ("violet", VIOLET),
    ("wheat", WHEAT),
    ("white", WHITE),
    ("white-smoke", WHITE_SMOKE),
    ("yellow", YELLOW),
    ("yellow-green", YELLOW_GREEN),
];

/// Lookup a CSS color by name (case-insensitive, supports kebab-case and snake_case).
pub fn from_name(name: &str) -> Option<Rgb565> {
    let mut i = 0;
    while i < CSS_PALETTE_TABLE.len() {
        let entry = CSS_PALETTE_TABLE[i].0;
        if entry.eq_ignore_ascii_case(name) {
            return Some(CSS_PALETTE_TABLE[i].1);
        }
        // Also compare normalized string without dashes or underscores
        let matches_normalized = entry.bytes().filter(|&b| b != b'-').eq(name
            .bytes()
            .filter(|&b| b != b'-' && b != b'_')
            .map(|b| b.to_ascii_lowercase()));
        if matches_normalized {
            return Some(CSS_PALETTE_TABLE[i].1);
        }
        i += 1;
    }
    None
}
