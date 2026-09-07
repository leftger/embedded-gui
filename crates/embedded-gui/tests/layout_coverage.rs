//! Coverage tests for linear/flex/grid layout arrangement.

use embedded_gui::layout::{
    Align, Axis, Constraint, GridLayout, GridPlacement, GridTrack, JustifyContent, LayoutItem,
    LinearLayout,
};
use embedded_gui::prelude::*;

#[test]
fn linear_layout_arrange_row_and_column() {
    let area = Rect::new(0, 0, 100, 100);
    let mut out = [Rect::empty(); 4];

    let row = LinearLayout::row()
        .with_gap(4)
        .with_padding(EdgeInsets::all(2))
        .with_justify(JustifyContent::Center)
        .with_cross_align(Align::Center);
    let n = row.arrange(area, 3, &mut out);
    assert_eq!(n, 3);
    assert!(out[0].y > 0);

    let column = LinearLayout::column()
        .with_gap(4)
        .with_padding(EdgeInsets::all(1));
    let n = column.arrange(area, 3, &mut out);
    assert_eq!(n, 3);
    assert!(out[0].x >= 1);
}

#[test]
fn layout_items_fixed_fill_percent_ratio_and_flex() {
    let area = Rect::new(0, 0, 100, 100);
    let items = [
        LayoutItem::fixed(20),
        LayoutItem::fill_weight(2),
        LayoutItem::percent(30),
        LayoutItem::ratio(1, 2),
        LayoutItem::min(10),
        LayoutItem::max(40),
        LayoutItem::flex(20),
        LayoutItem::rigid(5),
    ];
    let layout = LinearLayout::row()
        .with_gap(0)
        .with_padding(EdgeInsets::all(0));
    let mut out = [Rect::empty(); 8];
    let n = layout.arrange_items(area, &items, &mut out);
    assert_eq!(n, 8);
    assert!(out[0].w > 0);

    let flex = LinearLayout::row();
    let n = flex.arrange_items_flex(area, &items, &mut out, true, true);
    assert_eq!(n, 8);
    assert!(out[0].w > 0);

    for justify in [
        JustifyContent::Start,
        JustifyContent::Center,
        JustifyContent::End,
        JustifyContent::SpaceBetween,
        JustifyContent::SpaceAround,
        JustifyContent::SpaceEvenly,
    ] {
        let l = LinearLayout::row()
            .with_gap(0)
            .with_padding(EdgeInsets::all(0))
            .with_justify(justify);
        let n = l.arrange_items(
            area,
            &[LayoutItem::fixed(5), LayoutItem::fixed(5)],
            &mut out,
        );
        assert_eq!(n, 2);
    }
}

#[test]
fn grid_layout_arranges_px_fr_auto_and_spans() {
    let area = Rect::new(0, 0, 100, 80);
    let grid = GridLayout::<3, 2>::new(
        [GridTrack::Px(20), GridTrack::Fr(1), GridTrack::Auto],
        [GridTrack::Auto, GridTrack::Auto],
    )
    .with_col_gap(2)
    .with_row_gap(2)
    .with_padding(EdgeInsets::all(1));

    let placements = [
        GridPlacement::cell(0, 0),
        GridPlacement::cell(1, 0),
        GridPlacement::cell(2, 0),
        GridPlacement::span(0, 1, 2, 1),
    ];
    let mut out = [Rect::empty(); 4];
    let n = grid.arrange_cells(area, &placements, &mut out);
    assert_eq!(n, 4);
    assert!(out[0].w > 0);
    assert!(out[1].x > out[0].x);
    assert!(out[2].x > out[1].x);

    let uniform = GridLayout::<2, 2>::uniform(2, 2);
    let mut uout = [Rect::empty(); 4];
    assert_eq!(
        uniform.arrange_cells(
            area,
            &[GridPlacement::cell(0, 0), GridPlacement::cell(1, 1)],
            &mut uout
        ),
        2
    );
    let _ = Axis::Horizontal;
    let _ = Constraint::length(1);
}
