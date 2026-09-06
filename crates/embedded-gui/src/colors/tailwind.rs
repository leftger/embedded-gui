//! Tailwind CSS 3.4.1 color palette mapped directly to [`Rgb565`].
//!
//! Extracted from Bevy (MIT / Apache-2.0, Copyright (c) Tailwind Labs, Inc.).
//! Grouped by hue with numeric lightness scale (50 is light, 950 is dark).

use super::{Rgb565, from_raw565};

// --- Amber ---
pub const AMBER_50: Rgb565 = from_raw565(0xffdd);
pub const AMBER_100: Rgb565 = from_raw565(0xff98);
pub const AMBER_200: Rgb565 = from_raw565(0xff31);
pub const AMBER_300: Rgb565 = from_raw565(0xfe89);
pub const AMBER_400: Rgb565 = from_raw565(0xfde4);
pub const AMBER_500: Rgb565 = from_raw565(0xf4e1);
pub const AMBER_600: Rgb565 = from_raw565(0xdba0);
pub const AMBER_700: Rgb565 = from_raw565(0xb281);
pub const AMBER_800: Rgb565 = from_raw565(0x9201);
pub const AMBER_900: Rgb565 = from_raw565(0x79a1);
pub const AMBER_950: Rgb565 = from_raw565(0x40c0);

// --- Blue ---
pub const BLUE_50: Rgb565 = from_raw565(0xefbf);
pub const BLUE_100: Rgb565 = from_raw565(0xdf5f);
pub const BLUE_200: Rgb565 = from_raw565(0xbedf);
pub const BLUE_300: Rgb565 = from_raw565(0x963f);
pub const BLUE_400: Rgb565 = from_raw565(0x653f);
pub const BLUE_500: Rgb565 = from_raw565(0x3c1e);
pub const BLUE_600: Rgb565 = from_raw565(0x231d);
pub const BLUE_700: Rgb565 = from_raw565(0x1a7b);
pub const BLUE_800: Rgb565 = from_raw565(0x1a15);
pub const BLUE_900: Rgb565 = from_raw565(0x19d1);
pub const BLUE_950: Rgb565 = from_raw565(0x112a);

// --- Cyan ---
pub const CYAN_50: Rgb565 = from_raw565(0xefff);
pub const CYAN_100: Rgb565 = from_raw565(0xcfdf);
pub const CYAN_200: Rgb565 = from_raw565(0xa79f);
pub const CYAN_300: Rgb565 = from_raw565(0x675f);
pub const CYAN_400: Rgb565 = from_raw565(0x269d);
pub const CYAN_500: Rgb565 = from_raw565(0x05ba);
pub const CYAN_600: Rgb565 = from_raw565(0x0c96);
pub const CYAN_700: Rgb565 = from_raw565(0x0bb2);
pub const CYAN_800: Rgb565 = from_raw565(0x12ee);
pub const CYAN_900: Rgb565 = from_raw565(0x126c);
pub const CYAN_950: Rgb565 = from_raw565(0x0988);

// --- Emerald ---
pub const EMERALD_50: Rgb565 = from_raw565(0xeffe);
pub const EMERALD_100: Rgb565 = from_raw565(0xd7dc);
pub const EMERALD_200: Rgb565 = from_raw565(0xa79a);
pub const EMERALD_300: Rgb565 = from_raw565(0x6f36);
pub const EMERALD_400: Rgb565 = from_raw565(0x3693);
pub const EMERALD_500: Rgb565 = from_raw565(0x15d0);
pub const EMERALD_600: Rgb565 = from_raw565(0x04ad);
pub const EMERALD_700: Rgb565 = from_raw565(0x03ca);
pub const EMERALD_800: Rgb565 = from_raw565(0x02e8);
pub const EMERALD_900: Rgb565 = from_raw565(0x0267);
pub const EMERALD_950: Rgb565 = from_raw565(0x0164);

// --- Fuchsia ---
pub const FUCHSIA_50: Rgb565 = from_raw565(0xffbf);
pub const FUCHSIA_100: Rgb565 = from_raw565(0xff5f);
pub const FUCHSIA_200: Rgb565 = from_raw565(0xf69f);
pub const FUCHSIA_300: Rgb565 = from_raw565(0xf55f);
pub const FUCHSIA_400: Rgb565 = from_raw565(0xebdf);
pub const FUCHSIA_500: Rgb565 = from_raw565(0xda3d);
pub const FUCHSIA_600: Rgb565 = from_raw565(0xc13a);
pub const FUCHSIA_700: Rgb565 = from_raw565(0xa0f5);
pub const FUCHSIA_800: Rgb565 = from_raw565(0x80d1);
pub const FUCHSIA_900: Rgb565 = from_raw565(0x70ce);
pub const FUCHSIA_950: Rgb565 = from_raw565(0x4829);

// --- Gray ---
pub const GRAY_50: Rgb565 = from_raw565(0xffdf);
pub const GRAY_100: Rgb565 = from_raw565(0xf7be);
pub const GRAY_200: Rgb565 = from_raw565(0xe73d);
pub const GRAY_300: Rgb565 = from_raw565(0xd6bb);
pub const GRAY_400: Rgb565 = from_raw565(0x9d15);
pub const GRAY_500: Rgb565 = from_raw565(0x6b90);
pub const GRAY_600: Rgb565 = from_raw565(0x4aac);
pub const GRAY_700: Rgb565 = from_raw565(0x320a);
pub const GRAY_800: Rgb565 = from_raw565(0x1946);
pub const GRAY_900: Rgb565 = from_raw565(0x10c4);
pub const GRAY_950: Rgb565 = from_raw565(0x0022);

// --- Green ---
pub const GREEN_50: Rgb565 = from_raw565(0xf7fe);
pub const GREEN_100: Rgb565 = from_raw565(0xdffc);
pub const GREEN_200: Rgb565 = from_raw565(0xbfba);
pub const GREEN_300: Rgb565 = from_raw565(0x8775);
pub const GREEN_400: Rgb565 = from_raw565(0x4ef0);
pub const GREEN_500: Rgb565 = from_raw565(0x262b);
pub const GREEN_600: Rgb565 = from_raw565(0x1509);
pub const GREEN_700: Rgb565 = from_raw565(0x1407);
pub const GREEN_800: Rgb565 = from_raw565(0x1326);
pub const GREEN_900: Rgb565 = from_raw565(0x1285);
pub const GREEN_950: Rgb565 = from_raw565(0x0162);

// --- Indigo ---
pub const INDIGO_50: Rgb565 = from_raw565(0xef9f);
pub const INDIGO_100: Rgb565 = from_raw565(0xe73f);
pub const INDIGO_200: Rgb565 = from_raw565(0xc69f);
pub const INDIGO_300: Rgb565 = from_raw565(0xa5bf);
pub const INDIGO_400: Rgb565 = from_raw565(0x847f);
pub const INDIGO_500: Rgb565 = from_raw565(0x633e);
pub const INDIGO_600: Rgb565 = from_raw565(0x4a3c);
pub const INDIGO_700: Rgb565 = from_raw565(0x41d9);
pub const INDIGO_800: Rgb565 = from_raw565(0x3194);
pub const INDIGO_900: Rgb565 = from_raw565(0x3170);
pub const INDIGO_950: Rgb565 = from_raw565(0x18c9);

// --- Lime ---
pub const LIME_50: Rgb565 = from_raw565(0xf7fc);
pub const LIME_100: Rgb565 = from_raw565(0xeff9);
pub const LIME_200: Rgb565 = from_raw565(0xdfd3);
pub const LIME_300: Rgb565 = from_raw565(0xbf8c);
pub const LIME_400: Rgb565 = from_raw565(0xa726);
pub const LIME_500: Rgb565 = from_raw565(0x8662);
pub const LIME_600: Rgb565 = from_raw565(0x6501);
pub const LIME_700: Rgb565 = from_raw565(0x4be1);
pub const LIME_800: Rgb565 = from_raw565(0x3b02);
pub const LIME_900: Rgb565 = from_raw565(0x3282);
pub const LIME_950: Rgb565 = from_raw565(0x1960);

// --- Neutral ---
pub const NEUTRAL_50: Rgb565 = from_raw565(0xffdf);
pub const NEUTRAL_100: Rgb565 = from_raw565(0xf7be);
pub const NEUTRAL_200: Rgb565 = from_raw565(0xe73c);
pub const NEUTRAL_300: Rgb565 = from_raw565(0xd6ba);
pub const NEUTRAL_400: Rgb565 = from_raw565(0xa514);
pub const NEUTRAL_500: Rgb565 = from_raw565(0x738e);
pub const NEUTRAL_600: Rgb565 = from_raw565(0x528a);
pub const NEUTRAL_700: Rgb565 = from_raw565(0x4208);
pub const NEUTRAL_800: Rgb565 = from_raw565(0x2124);
pub const NEUTRAL_900: Rgb565 = from_raw565(0x10a2);
pub const NEUTRAL_950: Rgb565 = from_raw565(0x0841);

// --- Orange ---
pub const ORANGE_50: Rgb565 = from_raw565(0xffbd);
pub const ORANGE_100: Rgb565 = from_raw565(0xff7a);
pub const ORANGE_200: Rgb565 = from_raw565(0xfeb5);
pub const ORANGE_300: Rgb565 = from_raw565(0xfdce);
pub const ORANGE_400: Rgb565 = from_raw565(0xfc87);
pub const ORANGE_500: Rgb565 = from_raw565(0xfb82);
pub const ORANGE_600: Rgb565 = from_raw565(0xeac1);
pub const ORANGE_700: Rgb565 = from_raw565(0xc201);
pub const ORANGE_800: Rgb565 = from_raw565(0x99a2);
pub const ORANGE_900: Rgb565 = from_raw565(0x7962);
pub const ORANGE_950: Rgb565 = from_raw565(0x40a0);

// --- Pink ---
pub const PINK_50: Rgb565 = from_raw565(0xff9f);
pub const PINK_100: Rgb565 = from_raw565(0xff3e);
pub const PINK_200: Rgb565 = from_raw565(0xfe7d);
pub const PINK_300: Rgb565 = from_raw565(0xfd5a);
pub const PINK_400: Rgb565 = from_raw565(0xf396);
pub const PINK_500: Rgb565 = from_raw565(0xea53);
pub const PINK_600: Rgb565 = from_raw565(0xd92e);
pub const PINK_700: Rgb565 = from_raw565(0xb8cb);
pub const PINK_800: Rgb565 = from_raw565(0x98a9);
pub const PINK_900: Rgb565 = from_raw565(0x80c8);
pub const PINK_950: Rgb565 = from_raw565(0x5024);

// --- Purple ---
pub const PURPLE_50: Rgb565 = from_raw565(0xffbf);
pub const PURPLE_100: Rgb565 = from_raw565(0xf75f);
pub const PURPLE_200: Rgb565 = from_raw565(0xeebf);
pub const PURPLE_300: Rgb565 = from_raw565(0xddbf);
pub const PURPLE_400: Rgb565 = from_raw565(0xc43f);
pub const PURPLE_500: Rgb565 = from_raw565(0xaabe);
pub const PURPLE_600: Rgb565 = from_raw565(0x919d);
pub const PURPLE_700: Rgb565 = from_raw565(0x7919);
pub const PURPLE_800: Rgb565 = from_raw565(0x6915);
pub const PURPLE_900: Rgb565 = from_raw565(0x58f0);
pub const PURPLE_950: Rgb565 = from_raw565(0x382c);

// --- Red ---
pub const RED_50: Rgb565 = from_raw565(0xff9e);
pub const RED_100: Rgb565 = from_raw565(0xff1c);
pub const RED_200: Rgb565 = from_raw565(0xfe59);
pub const RED_300: Rgb565 = from_raw565(0xfd34);
pub const RED_400: Rgb565 = from_raw565(0xfb8e);
pub const RED_500: Rgb565 = from_raw565(0xea28);
pub const RED_600: Rgb565 = from_raw565(0xd924);
pub const RED_700: Rgb565 = from_raw565(0xb8e3);
pub const RED_800: Rgb565 = from_raw565(0x98c3);
pub const RED_900: Rgb565 = from_raw565(0x78e3);
pub const RED_950: Rgb565 = from_raw565(0x4041);

// --- Rose ---
pub const ROSE_50: Rgb565 = from_raw565(0xff9e);
pub const ROSE_100: Rgb565 = from_raw565(0xff3c);
pub const ROSE_200: Rgb565 = from_raw565(0xfe7a);
pub const ROSE_300: Rgb565 = from_raw565(0xfd35);
pub const ROSE_400: Rgb565 = from_raw565(0xfb90);
pub const ROSE_500: Rgb565 = from_raw565(0xf1eb);
pub const ROSE_600: Rgb565 = from_raw565(0xe0e9);
pub const ROSE_700: Rgb565 = from_raw565(0xb887);
pub const ROSE_800: Rgb565 = from_raw565(0x9887);
pub const ROSE_900: Rgb565 = from_raw565(0x8886);
pub const ROSE_950: Rgb565 = from_raw565(0x4823);

// --- Sky ---
pub const SKY_50: Rgb565 = from_raw565(0xf7df);
pub const SKY_100: Rgb565 = from_raw565(0xe79f);
pub const SKY_200: Rgb565 = from_raw565(0xbf3f);
pub const SKY_300: Rgb565 = from_raw565(0x7e9f);
pub const SKY_400: Rgb565 = from_raw565(0x3dff);
pub const SKY_500: Rgb565 = from_raw565(0x0d3d);
pub const SKY_600: Rgb565 = from_raw565(0x0438);
pub const SKY_700: Rgb565 = from_raw565(0x0354);
pub const SKY_800: Rgb565 = from_raw565(0x02d0);
pub const SKY_900: Rgb565 = from_raw565(0x0a4d);
pub const SKY_950: Rgb565 = from_raw565(0x0969);

// --- Slate ---
pub const SLATE_50: Rgb565 = from_raw565(0xffdf);
pub const SLATE_100: Rgb565 = from_raw565(0xf7bf);
pub const SLATE_200: Rgb565 = from_raw565(0xe75e);
pub const SLATE_300: Rgb565 = from_raw565(0xcebc);
pub const SLATE_400: Rgb565 = from_raw565(0x9517);
pub const SLATE_500: Rgb565 = from_raw565(0x63b1);
pub const SLATE_600: Rgb565 = from_raw565(0x42ad);
pub const SLATE_700: Rgb565 = from_raw565(0x320a);
pub const SLATE_800: Rgb565 = from_raw565(0x1947);
pub const SLATE_900: Rgb565 = from_raw565(0x08a5);
pub const SLATE_950: Rgb565 = from_raw565(0x0022);

// --- Stone ---
pub const STONE_50: Rgb565 = from_raw565(0xffdf);
pub const STONE_100: Rgb565 = from_raw565(0xf7be);
pub const STONE_200: Rgb565 = from_raw565(0xe73c);
pub const STONE_300: Rgb565 = from_raw565(0xd69a);
pub const STONE_400: Rgb565 = from_raw565(0xad13);
pub const STONE_500: Rgb565 = from_raw565(0x7b8d);
pub const STONE_600: Rgb565 = from_raw565(0x5289);
pub const STONE_700: Rgb565 = from_raw565(0x4207);
pub const STONE_800: Rgb565 = from_raw565(0x2924);
pub const STONE_900: Rgb565 = from_raw565(0x18c2);
pub const STONE_950: Rgb565 = from_raw565(0x0841);

// --- Teal ---
pub const TEAL_50: Rgb565 = from_raw565(0xf7ff);
pub const TEAL_100: Rgb565 = from_raw565(0xcfde);
pub const TEAL_200: Rgb565 = from_raw565(0x9fbc);
pub const TEAL_300: Rgb565 = from_raw565(0x5f5a);
pub const TEAL_400: Rgb565 = from_raw565(0x2eb7);
pub const TEAL_500: Rgb565 = from_raw565(0x15d4);
pub const TEAL_600: Rgb565 = from_raw565(0x0cb1);
pub const TEAL_700: Rgb565 = from_raw565(0x0bad);
pub const TEAL_800: Rgb565 = from_raw565(0x12eb);
pub const TEAL_900: Rgb565 = from_raw565(0x1269);
pub const TEAL_950: Rgb565 = from_raw565(0x0165);

// --- Violet ---
pub const VIOLET_50: Rgb565 = from_raw565(0xf79f);
pub const VIOLET_100: Rgb565 = from_raw565(0xef5f);
pub const VIOLET_200: Rgb565 = from_raw565(0xdebf);
pub const VIOLET_300: Rgb565 = from_raw565(0xc5bf);
pub const VIOLET_400: Rgb565 = from_raw565(0xa45f);
pub const VIOLET_500: Rgb565 = from_raw565(0x8afe);
pub const VIOLET_600: Rgb565 = from_raw565(0x79dd);
pub const VIOLET_700: Rgb565 = from_raw565(0x695b);
pub const VIOLET_800: Rgb565 = from_raw565(0x5916);
pub const VIOLET_900: Rgb565 = from_raw565(0x48f2);
pub const VIOLET_950: Rgb565 = from_raw565(0x288c);

// --- Yellow ---
pub const YELLOW_50: Rgb565 = from_raw565(0xfffd);
pub const YELLOW_100: Rgb565 = from_raw565(0xffd8);
pub const YELLOW_200: Rgb565 = from_raw565(0xff91);
pub const YELLOW_300: Rgb565 = from_raw565(0xff08);
pub const YELLOW_400: Rgb565 = from_raw565(0xfe62);
pub const YELLOW_500: Rgb565 = from_raw565(0xed81);
pub const YELLOW_600: Rgb565 = from_raw565(0xcc40);
pub const YELLOW_700: Rgb565 = from_raw565(0xa300);
pub const YELLOW_800: Rgb565 = from_raw565(0x8261);
pub const YELLOW_900: Rgb565 = from_raw565(0x71e2);
pub const YELLOW_950: Rgb565 = from_raw565(0x4100);

// --- Zinc ---
pub const ZINC_50: Rgb565 = from_raw565(0xffdf);
pub const ZINC_100: Rgb565 = from_raw565(0xf7be);
pub const ZINC_200: Rgb565 = from_raw565(0xe73c);
pub const ZINC_300: Rgb565 = from_raw565(0xd6bb);
pub const ZINC_400: Rgb565 = from_raw565(0xa515);
pub const ZINC_500: Rgb565 = from_raw565(0x738f);
pub const ZINC_600: Rgb565 = from_raw565(0x528b);
pub const ZINC_700: Rgb565 = from_raw565(0x39e8);
pub const ZINC_800: Rgb565 = from_raw565(0x2125);
pub const ZINC_900: Rgb565 = from_raw565(0x18c3);
pub const ZINC_950: Rgb565 = from_raw565(0x0841);

const TAILWIND_TABLE: &[(&str, Rgb565)] = &[
    ("amber-100", AMBER_100),
    ("amber-200", AMBER_200),
    ("amber-300", AMBER_300),
    ("amber-400", AMBER_400),
    ("amber-50", AMBER_50),
    ("amber-500", AMBER_500),
    ("amber-600", AMBER_600),
    ("amber-700", AMBER_700),
    ("amber-800", AMBER_800),
    ("amber-900", AMBER_900),
    ("amber-950", AMBER_950),
    ("blue-100", BLUE_100),
    ("blue-200", BLUE_200),
    ("blue-300", BLUE_300),
    ("blue-400", BLUE_400),
    ("blue-50", BLUE_50),
    ("blue-500", BLUE_500),
    ("blue-600", BLUE_600),
    ("blue-700", BLUE_700),
    ("blue-800", BLUE_800),
    ("blue-900", BLUE_900),
    ("blue-950", BLUE_950),
    ("cyan-100", CYAN_100),
    ("cyan-200", CYAN_200),
    ("cyan-300", CYAN_300),
    ("cyan-400", CYAN_400),
    ("cyan-50", CYAN_50),
    ("cyan-500", CYAN_500),
    ("cyan-600", CYAN_600),
    ("cyan-700", CYAN_700),
    ("cyan-800", CYAN_800),
    ("cyan-900", CYAN_900),
    ("cyan-950", CYAN_950),
    ("emerald-100", EMERALD_100),
    ("emerald-200", EMERALD_200),
    ("emerald-300", EMERALD_300),
    ("emerald-400", EMERALD_400),
    ("emerald-50", EMERALD_50),
    ("emerald-500", EMERALD_500),
    ("emerald-600", EMERALD_600),
    ("emerald-700", EMERALD_700),
    ("emerald-800", EMERALD_800),
    ("emerald-900", EMERALD_900),
    ("emerald-950", EMERALD_950),
    ("fuchsia-100", FUCHSIA_100),
    ("fuchsia-200", FUCHSIA_200),
    ("fuchsia-300", FUCHSIA_300),
    ("fuchsia-400", FUCHSIA_400),
    ("fuchsia-50", FUCHSIA_50),
    ("fuchsia-500", FUCHSIA_500),
    ("fuchsia-600", FUCHSIA_600),
    ("fuchsia-700", FUCHSIA_700),
    ("fuchsia-800", FUCHSIA_800),
    ("fuchsia-900", FUCHSIA_900),
    ("fuchsia-950", FUCHSIA_950),
    ("gray-100", GRAY_100),
    ("gray-200", GRAY_200),
    ("gray-300", GRAY_300),
    ("gray-400", GRAY_400),
    ("gray-50", GRAY_50),
    ("gray-500", GRAY_500),
    ("gray-600", GRAY_600),
    ("gray-700", GRAY_700),
    ("gray-800", GRAY_800),
    ("gray-900", GRAY_900),
    ("gray-950", GRAY_950),
    ("green-100", GREEN_100),
    ("green-200", GREEN_200),
    ("green-300", GREEN_300),
    ("green-400", GREEN_400),
    ("green-50", GREEN_50),
    ("green-500", GREEN_500),
    ("green-600", GREEN_600),
    ("green-700", GREEN_700),
    ("green-800", GREEN_800),
    ("green-900", GREEN_900),
    ("green-950", GREEN_950),
    ("indigo-100", INDIGO_100),
    ("indigo-200", INDIGO_200),
    ("indigo-300", INDIGO_300),
    ("indigo-400", INDIGO_400),
    ("indigo-50", INDIGO_50),
    ("indigo-500", INDIGO_500),
    ("indigo-600", INDIGO_600),
    ("indigo-700", INDIGO_700),
    ("indigo-800", INDIGO_800),
    ("indigo-900", INDIGO_900),
    ("indigo-950", INDIGO_950),
    ("lime-100", LIME_100),
    ("lime-200", LIME_200),
    ("lime-300", LIME_300),
    ("lime-400", LIME_400),
    ("lime-50", LIME_50),
    ("lime-500", LIME_500),
    ("lime-600", LIME_600),
    ("lime-700", LIME_700),
    ("lime-800", LIME_800),
    ("lime-900", LIME_900),
    ("lime-950", LIME_950),
    ("neutral-100", NEUTRAL_100),
    ("neutral-200", NEUTRAL_200),
    ("neutral-300", NEUTRAL_300),
    ("neutral-400", NEUTRAL_400),
    ("neutral-50", NEUTRAL_50),
    ("neutral-500", NEUTRAL_500),
    ("neutral-600", NEUTRAL_600),
    ("neutral-700", NEUTRAL_700),
    ("neutral-800", NEUTRAL_800),
    ("neutral-900", NEUTRAL_900),
    ("neutral-950", NEUTRAL_950),
    ("orange-100", ORANGE_100),
    ("orange-200", ORANGE_200),
    ("orange-300", ORANGE_300),
    ("orange-400", ORANGE_400),
    ("orange-50", ORANGE_50),
    ("orange-500", ORANGE_500),
    ("orange-600", ORANGE_600),
    ("orange-700", ORANGE_700),
    ("orange-800", ORANGE_800),
    ("orange-900", ORANGE_900),
    ("orange-950", ORANGE_950),
    ("pink-100", PINK_100),
    ("pink-200", PINK_200),
    ("pink-300", PINK_300),
    ("pink-400", PINK_400),
    ("pink-50", PINK_50),
    ("pink-500", PINK_500),
    ("pink-600", PINK_600),
    ("pink-700", PINK_700),
    ("pink-800", PINK_800),
    ("pink-900", PINK_900),
    ("pink-950", PINK_950),
    ("purple-100", PURPLE_100),
    ("purple-200", PURPLE_200),
    ("purple-300", PURPLE_300),
    ("purple-400", PURPLE_400),
    ("purple-50", PURPLE_50),
    ("purple-500", PURPLE_500),
    ("purple-600", PURPLE_600),
    ("purple-700", PURPLE_700),
    ("purple-800", PURPLE_800),
    ("purple-900", PURPLE_900),
    ("purple-950", PURPLE_950),
    ("red-100", RED_100),
    ("red-200", RED_200),
    ("red-300", RED_300),
    ("red-400", RED_400),
    ("red-50", RED_50),
    ("red-500", RED_500),
    ("red-600", RED_600),
    ("red-700", RED_700),
    ("red-800", RED_800),
    ("red-900", RED_900),
    ("red-950", RED_950),
    ("rose-100", ROSE_100),
    ("rose-200", ROSE_200),
    ("rose-300", ROSE_300),
    ("rose-400", ROSE_400),
    ("rose-50", ROSE_50),
    ("rose-500", ROSE_500),
    ("rose-600", ROSE_600),
    ("rose-700", ROSE_700),
    ("rose-800", ROSE_800),
    ("rose-900", ROSE_900),
    ("rose-950", ROSE_950),
    ("sky-100", SKY_100),
    ("sky-200", SKY_200),
    ("sky-300", SKY_300),
    ("sky-400", SKY_400),
    ("sky-50", SKY_50),
    ("sky-500", SKY_500),
    ("sky-600", SKY_600),
    ("sky-700", SKY_700),
    ("sky-800", SKY_800),
    ("sky-900", SKY_900),
    ("sky-950", SKY_950),
    ("slate-100", SLATE_100),
    ("slate-200", SLATE_200),
    ("slate-300", SLATE_300),
    ("slate-400", SLATE_400),
    ("slate-50", SLATE_50),
    ("slate-500", SLATE_500),
    ("slate-600", SLATE_600),
    ("slate-700", SLATE_700),
    ("slate-800", SLATE_800),
    ("slate-900", SLATE_900),
    ("slate-950", SLATE_950),
    ("stone-100", STONE_100),
    ("stone-200", STONE_200),
    ("stone-300", STONE_300),
    ("stone-400", STONE_400),
    ("stone-50", STONE_50),
    ("stone-500", STONE_500),
    ("stone-600", STONE_600),
    ("stone-700", STONE_700),
    ("stone-800", STONE_800),
    ("stone-900", STONE_900),
    ("stone-950", STONE_950),
    ("teal-100", TEAL_100),
    ("teal-200", TEAL_200),
    ("teal-300", TEAL_300),
    ("teal-400", TEAL_400),
    ("teal-50", TEAL_50),
    ("teal-500", TEAL_500),
    ("teal-600", TEAL_600),
    ("teal-700", TEAL_700),
    ("teal-800", TEAL_800),
    ("teal-900", TEAL_900),
    ("teal-950", TEAL_950),
    ("violet-100", VIOLET_100),
    ("violet-200", VIOLET_200),
    ("violet-300", VIOLET_300),
    ("violet-400", VIOLET_400),
    ("violet-50", VIOLET_50),
    ("violet-500", VIOLET_500),
    ("violet-600", VIOLET_600),
    ("violet-700", VIOLET_700),
    ("violet-800", VIOLET_800),
    ("violet-900", VIOLET_900),
    ("violet-950", VIOLET_950),
    ("yellow-100", YELLOW_100),
    ("yellow-200", YELLOW_200),
    ("yellow-300", YELLOW_300),
    ("yellow-400", YELLOW_400),
    ("yellow-50", YELLOW_50),
    ("yellow-500", YELLOW_500),
    ("yellow-600", YELLOW_600),
    ("yellow-700", YELLOW_700),
    ("yellow-800", YELLOW_800),
    ("yellow-900", YELLOW_900),
    ("yellow-950", YELLOW_950),
    ("zinc-100", ZINC_100),
    ("zinc-200", ZINC_200),
    ("zinc-300", ZINC_300),
    ("zinc-400", ZINC_400),
    ("zinc-50", ZINC_50),
    ("zinc-500", ZINC_500),
    ("zinc-600", ZINC_600),
    ("zinc-700", ZINC_700),
    ("zinc-800", ZINC_800),
    ("zinc-900", ZINC_900),
    ("zinc-950", ZINC_950),
];

/// Look up a Tailwind color token by name (e.g. `"slate-500"` or `"SLATE_500"`).
pub fn from_name(name: &str) -> Option<Rgb565> {
    let mut i = 0;
    while i < TAILWIND_TABLE.len() {
        let (entry, color) = TAILWIND_TABLE[i];
        if entry.eq_ignore_ascii_case(name) {
            return Some(color);
        }
        i += 1;
    }
    // Also match with underscores normalized to hyphens
    if name.contains("_") {
        for &(entry, color) in TAILWIND_TABLE {
            if entry.len() == name.len() {
                let mut matched = true;
                let mut nb = name.bytes();
                let mut eb = entry.bytes();
                while let (Some(n), Some(e)) = (nb.next(), eb.next()) {
                    let n_norm = if n == b"_"[0] {
                        b"-"[0]
                    } else {
                        n.to_ascii_lowercase()
                    };
                    if n_norm != e {
                        matched = false;
                        break;
                    }
                }
                if matched {
                    return Some(color);
                }
            }
        }
    }
    None
}
