use criterion::{Criterion, criterion_group, criterion_main};
use embedded_gui::framebuffer::Framebuffer;
use embedded_gui::prelude::*;
use embedded_gui::{PropertyKey, PropertyValue};

fn bench_widget_tree_creation(c: &mut Criterion) {
    c.bench_function("gui_widget_tree_creation", |b| {
        b.iter(|| {
            let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 320, 240));
            let panel = gui
                .add_panel(Rect::new(0, 0, 320, 240), Style::panel())
                .unwrap();
            let label = gui
                .add_label(Rect::new(10, 10, 100, 20), "Header", Style::label())
                .unwrap();
            let button = gui
                .add_button(Rect::new(10, 40, 80, 30), "Click", Style::button())
                .unwrap();
            let progress = gui
                .add_progress_bar(Rect::new(10, 80, 150, 10), 0.4, Style::progress())
                .unwrap();
            let _ = gui.add_child(panel, label);
            let _ = gui.add_child(panel, button);
            let _ = gui.add_child(panel, progress);
            gui
        })
    });
}

fn bench_full_frame_rendering(c: &mut Criterion) {
    const SIZE: usize = 320 * 240;
    let mut fb = Framebuffer::<SIZE>::new(320, 240);
    let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 320, 240));

    let panel = gui
        .add_panel(Rect::new(0, 0, 320, 240), Style::panel())
        .unwrap();
    let label = gui
        .add_label(Rect::new(10, 10, 100, 20), "Benchmark", Style::label())
        .unwrap();
    let button = gui
        .add_button(Rect::new(10, 40, 80, 30), "Action", Style::button())
        .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(10, 80, 150, 10), 0.75, Style::progress())
        .unwrap();
    let _ = gui.add_child(panel, label);
    let _ = gui.add_child(panel, button);
    let _ = gui.add_child(panel, progress);

    c.bench_function("gui_full_frame_render_320x240", |b| {
        b.iter(|| {
            let _ = gui.render(&mut fb);
        })
    });
}

fn bench_property_mutations(c: &mut Criterion) {
    let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 320, 240));
    let progress_id = gui
        .add_progress_bar(Rect::new(10, 10, 150, 10), 0.0, Style::progress())
        .unwrap();

    let mut val = 0.0f32;
    c.bench_function("gui_generic_property_mutation", |b| {
        b.iter(|| {
            val = (val + 0.01) % 1.0;
            let _ =
                gui.set_widget_property(progress_id, PropertyKey::Value, PropertyValue::Float(val));
        })
    });
}

fn bench_layout_arrangement(c: &mut Criterion) {
    let layout = LinearLayout::column().with_gap(4);
    let mut out = [Rect::empty(); 16];

    c.bench_function("gui_linear_layout_16_items", |b| {
        b.iter(|| {
            layout.arrange(Rect::new(0, 0, 320, 240), 16, &mut out);
        })
    });
}

criterion_group!(
    benches,
    bench_widget_tree_creation,
    bench_full_frame_rendering,
    bench_property_mutations,
    bench_layout_arrangement
);
criterion_main!(benches);
