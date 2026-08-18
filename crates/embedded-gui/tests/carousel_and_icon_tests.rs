//! Pixel-level behavior of the carousel and composite icon widgets.

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::prelude::*;

const ITEMS: [&str; 5] = ["ALPHA", "BRAVO", "CHARLIE", "DELTA", "ECHO"];

/// A 12x8 checkerboard-ish glyph: two ink rows with a gap between them.
const BARS: [u8; 16] = [
    0xFF, 0xF0, 0x00, 0x00, 0xFF, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

fn white_on_black() -> Style {
    let mut style = Style::label();
    style.background = Some(Rgb565::new(0, 0, 0));
    style.text = Rgb565::new(31, 63, 31);
    style.foreground = Rgb565::new(31, 63, 31);
    style.accent = Rgb565::new(0, 63, 0);
    style
}

fn render(gui: &mut GuiContext<'_, 8, 4, 8>, w: u32, h: u32) -> TestBuffer {
    let mut buffer = TestBuffer::new(w, h);
    gui.render(&mut buffer).unwrap();
    buffer
}

fn row_brightness(buffer: &TestBuffer, width: u32, y: i32) -> u16 {
    (0..width as i32)
        .filter_map(|x| buffer.pixel_at(x, y))
        .map(|px| u16::from(px.r()) + u16::from(px.g()) + u16::from(px.b()))
        .max()
        .unwrap_or(0)
}

#[test]
fn carousel_rows_dim_with_distance_from_the_selection() {
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
    gui.add_carousel(
        Rect::new(0, 0, 96, 64),
        &ITEMS,
        2,
        CarouselSpec {
            item_step: 16,
            visible_slots: 5,
            fade_edges: false,
            ..CarouselSpec::default()
        },
        white_on_black(),
    )
    .unwrap();
    let buffer = render(&mut gui, 96, 64);

    let brightest_near = |center: i32| {
        (center - 5..=center + 5)
            .map(|y| row_brightness(&buffer, 96, y))
            .max()
            .unwrap_or(0)
    };
    let selected = brightest_near(32);
    let neighbour = brightest_near(48);
    let far = brightest_near(64 - 1);

    assert!(
        selected > neighbour,
        "selected row {selected} not brighter than its neighbour {neighbour}"
    );
    assert!(
        neighbour > far,
        "falloff did not continue: {neighbour} vs {far}"
    );
}

#[test]
fn carousel_wraps_around_the_item_list() {
    // With the first item selected, the row above it shows the last item, so
    // the list reads as an endless loop rather than a bounded scroll.
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
    gui.add_carousel(
        Rect::new(0, 0, 96, 64),
        &ITEMS,
        0,
        CarouselSpec {
            item_step: 16,
            visible_slots: 3,
            fade_edges: false,
            ..CarouselSpec::default()
        },
        white_on_black(),
    )
    .unwrap();
    let buffer = render(&mut gui, 96, 64);
    assert!(
        row_brightness(&buffer, 96, 16) > 0,
        "the wrapped row above the selection was not drawn"
    );
}

#[test]
fn carousel_shift_moves_rows_by_whole_pixels() {
    let base = {
        let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
        gui.add_carousel(
            Rect::new(0, 0, 96, 64),
            &ITEMS,
            2,
            CarouselSpec {
                item_step: 16,
                visible_slots: 3,
                fade_edges: false,
                ..CarouselSpec::default()
            },
            white_on_black(),
        )
        .unwrap();
        render(&mut gui, 96, 64)
    };
    let shifted = {
        let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
        gui.add_carousel(
            Rect::new(0, 0, 96, 64),
            &ITEMS,
            2,
            CarouselSpec {
                item_step: 16,
                visible_slots: 3,
                shift: 4,
                fade_edges: false,
                ..CarouselSpec::default()
            },
            white_on_black(),
        )
        .unwrap();
        render(&mut gui, 96, 64)
    };

    for y in 0..60 {
        assert_eq!(
            row_brightness(&base, 96, y),
            row_brightness(&shifted, 96, y + 4),
            "row {y} did not move down by the 4px shift"
        );
    }
}

#[test]
fn hidden_composite_icon_parts_leave_the_backdrop_alone() {
    let parts_all_visible = [
        IconPart::new(MonoBitmap::new(12, 8, &BARS), 0, 0),
        IconPart::new(MonoBitmap::new(12, 8, &BARS), 0, 8),
    ];
    let parts_one_hidden = [
        parts_all_visible[0],
        parts_all_visible[1].with_visible(false),
    ];

    let count_ink = |parts: &[IconPart<'_>]| {
        let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 32, 32));
        gui.add_composite_icon(
            Rect::new(0, 0, 32, 32),
            parts,
            CompositeIconSpec::default(),
            white_on_black(),
        )
        .unwrap();
        let buffer = render(&mut gui, 32, 32);
        (0..32)
            .flat_map(|y| (0..32).map(move |x| (x, y)))
            .filter(|(x, y)| {
                buffer
                    .pixel_at(*x, *y)
                    .is_some_and(|px| px == Rgb565::new(31, 63, 31))
            })
            .count()
    };

    let both = count_ink(&parts_all_visible);
    let one = count_ink(&parts_one_hidden);
    assert!(both > 0, "no ink drawn for a fully visible icon");
    assert_eq!(
        both,
        one * 2,
        "hiding one of two identical parts should halve the ink"
    );
}

#[test]
fn composite_icon_scale_multiplies_ink_area() {
    let parts = [IconPart::new(MonoBitmap::new(12, 8, &BARS), 0, 0)];
    let ink_at = |scale: u8| {
        let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 64, 64));
        gui.add_composite_icon(
            Rect::new(0, 0, 64, 64),
            &parts,
            CompositeIconSpec {
                scale,
                ..CompositeIconSpec::default()
            },
            white_on_black(),
        )
        .unwrap();
        let buffer = render(&mut gui, 64, 64);
        (0..64)
            .flat_map(|y| (0..64).map(move |x| (x, y)))
            .filter(|(x, y)| {
                buffer
                    .pixel_at(*x, *y)
                    .is_some_and(|px| px == Rgb565::new(31, 63, 31))
            })
            .count()
    };

    assert_eq!(ink_at(2), ink_at(1) * 4);
}

#[test]
fn set_carousel_selected_recenters_the_bright_row() {
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
    let id = gui
        .add_carousel(
            Rect::new(0, 0, 96, 64),
            &ITEMS,
            0,
            CarouselSpec {
                item_step: 16,
                visible_slots: 5,
                fade_edges: false,
                ..CarouselSpec::default()
            },
            white_on_black(),
        )
        .unwrap();

    assert_eq!(gui.carousel_selected(id), Some(0));
    gui.set_carousel_selected(id, 3).unwrap();
    assert_eq!(gui.carousel_selected(id), Some(3));
    // Clamps past the end rather than erroring.
    gui.set_carousel_selected(id, 99).unwrap();
    assert_eq!(gui.carousel_selected(id), Some(ITEMS.len() - 1));
}

#[test]
fn set_carousel_shift_matches_a_spec_authored_shift() {
    let authored = {
        let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
        gui.add_carousel(
            Rect::new(0, 0, 96, 64),
            &ITEMS,
            2,
            CarouselSpec {
                item_step: 16,
                visible_slots: 3,
                shift: 5,
                fade_edges: false,
                ..CarouselSpec::default()
            },
            white_on_black(),
        )
        .unwrap();
        render(&mut gui, 96, 64)
    };
    let mutated = {
        let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
        let id = gui
            .add_carousel(
                Rect::new(0, 0, 96, 64),
                &ITEMS,
                2,
                CarouselSpec {
                    item_step: 16,
                    visible_slots: 3,
                    fade_edges: false,
                    ..CarouselSpec::default()
                },
                white_on_black(),
            )
            .unwrap();
        gui.set_carousel_shift(id, 5).unwrap();
        render(&mut gui, 96, 64)
    };

    for y in 0..64 {
        assert_eq!(
            row_brightness(&authored, 96, y),
            row_brightness(&mutated, 96, y),
            "runtime shift diverged from an authored shift at row {y}"
        );
    }
}

#[test]
fn swapping_icon_parts_toggles_visibility_at_runtime() {
    // Firmware owns a mutable copy of the baked parts and swaps the slice to
    // reveal or hide a layer.
    let visible = [
        IconPart::new(MonoBitmap::new(12, 8, &BARS), 0, 0),
        IconPart::new(MonoBitmap::new(12, 8, &BARS), 0, 8),
    ];
    let hidden = [visible[0], visible[1].with_visible(false)];

    let ink = |gui: &mut GuiContext<'_, 8, 4, 8>| {
        let buffer = render(gui, 32, 32);
        (0..32)
            .flat_map(|y| (0..32).map(move |x| (x, y)))
            .filter(|(x, y)| {
                buffer
                    .pixel_at(*x, *y)
                    .is_some_and(|px| px == Rgb565::new(31, 63, 31))
            })
            .count()
    };

    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 32, 32));
    let id = gui
        .add_composite_icon(
            Rect::new(0, 0, 32, 32),
            &visible,
            CompositeIconSpec::default(),
            white_on_black(),
        )
        .unwrap();
    let both = ink(&mut gui);

    gui.set_composite_icon_parts(id, &hidden).unwrap();
    let one = ink(&mut gui);

    assert!(both > 0, "no ink for fully visible icon");
    assert_eq!(
        both,
        one * 2,
        "swapping to a hidden part did not halve the ink"
    );
}

#[test]
fn per_part_tint_overrides_the_icon_ink() {
    let gold = Rgb565::new(31, 40, 0);
    let parts = [IconPart::new(MonoBitmap::new(12, 8, &BARS), 0, 0).with_tint(gold)];
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 32, 32));
    gui.add_composite_icon(
        Rect::new(0, 0, 32, 32),
        &parts,
        CompositeIconSpec::default(),
        white_on_black(),
    )
    .unwrap();
    let buffer = render(&mut gui, 32, 32);

    let gold_pixels = (0..32)
        .flat_map(|y| (0..32).map(move |x| (x, y)))
        .filter(|(x, y)| buffer.pixel_at(*x, *y).is_some_and(|px| px == gold))
        .count();
    assert!(gold_pixels > 0, "the part's own tint was not used");
}
