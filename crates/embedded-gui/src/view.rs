use crate::{
    GuiContext,
    context::types::GuiError,
    geometry::{EdgeInsets, Rect},
    layout::{Align, Axis, JustifyContent, LinearLayout},
    style::WidgetStyle,
    widget::WidgetId,
};
use embedded_graphics_core::pixelcolor::Rgb565;
use heapless::Vec;

/// Trait implemented by views and components that declaratively build UI trees in pure Rust.
pub trait Render {
    /// Declaratively builds the view into the given context and returns the root widget ID.
    fn render<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
        &self,
        cx: &mut ViewContext<'a, 'ctx, NODES, EVENTS, DIRTY>,
    ) -> Result<WidgetId, GuiError>;
}

/// Declarative context providing builder methods for UI components and layouts.
pub struct ViewContext<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize> {
    pub ctx: &'ctx mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    pub bounds: Rect,
}

impl<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    ViewContext<'a, 'ctx, NODES, EVENTS, DIRTY>
{
    /// Creates a new view context for the given GUI context and viewport bounds.
    pub fn new(
        ctx: &'ctx mut GuiContext<'a, NODES, EVENTS, DIRTY>,
        bounds: impl Into<Rect>,
    ) -> Self {
        Self {
            ctx,
            bounds: bounds.into(),
        }
    }

    /// Spawns a column layout builder.
    pub fn column<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        let flex = FlexBuilder::new(self.ctx, self.bounds, Axis::Vertical);
        builder(flex)
    }

    /// Spawns a row layout builder.
    pub fn row<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        let flex = FlexBuilder::new(self.ctx, self.bounds, Axis::Horizontal);
        builder(flex)
    }

    /// Spawns a flex-direction agnostic container (div) builder with default row orientation.
    pub fn div<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        self.row(builder)
    }

    /// Spawns an explicit flex-row container builder.
    pub fn flex_row<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        self.row(builder)
    }

    /// Spawns an explicit flex-column container builder.
    pub fn flex_col<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        self.column(builder)
    }

    /// Spawns a themed label widget.
    pub fn label(&mut self, text: &'a str) -> Result<WidgetId, GuiError> {
        self.ctx.add_themed_label(self.bounds, text)
    }

    /// Spawns a styled label widget.
    pub fn styled_label(
        &mut self,
        text: &'a str,
        style: impl Into<WidgetStyle>,
    ) -> Result<WidgetId, GuiError> {
        self.ctx.add_label(self.bounds, text, style)
    }

    /// Spawns a themed button widget.
    pub fn button(&mut self, text: &'a str) -> Result<WidgetId, GuiError> {
        self.ctx.add_themed_button(self.bounds, text)
    }

    /// Spawns a themed panel container.
    pub fn panel<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(&mut ViewContext<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        let panel_id = self.ctx.add_themed_panel(self.bounds)?;
        let mut inner_cx = ViewContext {
            ctx: self.ctx,
            bounds: self.bounds,
        };
        let child_id = builder(&mut inner_cx)?;
        self.ctx.add_child(panel_id, child_id)?;
        Ok(panel_id)
    }

    /// Spawns a themed toggle widget.
    #[cfg(feature = "rich-widgets")]
    pub fn toggle(&mut self, label: &'a str, on: bool) -> Result<WidgetId, GuiError> {
        self.ctx.add_themed_toggle(self.bounds, label, on)
    }

    /// Spawns a themed slider widget.
    #[cfg(feature = "rich-widgets")]
    pub fn slider(&mut self, value: f32, min: f32, max: f32) -> Result<WidgetId, GuiError> {
        self.ctx.add_themed_slider(self.bounds, value, min, max)
    }

    /// Spawns a themed progress bar widget.
    #[cfg(feature = "rich-widgets")]
    pub fn progress_bar(&mut self, value: f32) -> Result<WidgetId, GuiError> {
        self.ctx.add_themed_progress_bar(self.bounds, value)
    }
}

/// Builder for linear (row/column) flex layouts in pure Rust declarative code.
pub struct FlexBuilder<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize> {
    ctx: &'ctx mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    bounds: Rect,
    layout: LinearLayout,
    children: Vec<WidgetId, 16>,
    style_override: Option<WidgetStyle>,
}

impl<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    FlexBuilder<'a, 'ctx, NODES, EVENTS, DIRTY>
{
    pub fn new(
        ctx: &'ctx mut GuiContext<'a, NODES, EVENTS, DIRTY>,
        bounds: Rect,
        axis: Axis,
    ) -> Self {
        let mut layout = match axis {
            Axis::Horizontal => LinearLayout::row(),
            Axis::Vertical => LinearLayout::column(),
        };
        layout.gap = 4;
        Self {
            ctx,
            bounds,
            layout,
            children: Vec::new(),
            style_override: None,
        }
    }

    /// Configures container width and height explicitly.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.bounds.w = width;
        self.bounds.h = height;
        self
    }

    /// Configures container width explicitly.
    pub fn width(mut self, width: u32) -> Self {
        self.bounds.w = width;
        self
    }

    /// Configures container height explicitly.
    pub fn height(mut self, height: u32) -> Self {
        self.bounds.h = height;
        self
    }

    /// Sets flex layout direction (Row or Column).
    pub fn flex_direction(mut self, axis: Axis) -> Self {
        self.layout.axis = axis;
        self
    }

    /// Configures container background color.
    pub fn background(mut self, color: Rgb565) -> Self {
        let style = self.style_override.unwrap_or_else(WidgetStyle::panel);
        self.style_override = Some(style.with_bg(color));
        self
    }

    /// Alias for `background`.
    pub fn bg(self, color: Rgb565) -> Self {
        self.background(color)
    }

    /// Configures container border width and color.
    pub fn border(mut self, width: u8, color: Rgb565) -> Self {
        let mut style = self.style_override.unwrap_or_else(WidgetStyle::panel);
        style.normal.border = crate::style::Border { color, width };
        self.style_override = Some(style);
        self
    }

    /// Configures container corner radius.
    pub fn corner_radius(mut self, radius: u8) -> Self {
        let mut style = self.style_override.unwrap_or_else(WidgetStyle::panel);
        style.normal.corner_radius = radius;
        self.style_override = Some(style);
        self
    }

    /// Configures gap spacing between children.
    pub fn gap(mut self, gap: u16) -> Self {
        self.layout.gap = gap;
        self
    }

    /// Configures padding around children using any CSS inset shorthand (`10`, `(4, 8)`, etc.).
    pub fn padding(mut self, padding: impl Into<EdgeInsets>) -> Self {
        self.layout.padding = padding.into();
        self
    }

    /// Configures cross-axis alignment.
    pub fn cross_align(mut self, align: Align) -> Self {
        self.layout.cross_align = align;
        self
    }

    /// Configures cross-axis alignment (Guillotine / CSS alias for `cross_align`).
    pub fn align_items(self, align: Align) -> Self {
        self.cross_align(align)
    }

    /// Configures main-axis content justification.
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.layout.justify = justify;
        self
    }

    /// Configures main-axis content justification (Guillotine / CSS alias for `justify`).
    pub fn justify_content(self, justify: JustifyContent) -> Self {
        self.justify(justify)
    }

    /// Appends a child element by calling a builder closure.
    pub fn child<F>(mut self, f: F) -> Result<Self, GuiError>
    where
        F: FnOnce(&mut ViewContext<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        let mut child_cx = ViewContext {
            ctx: self.ctx,
            bounds: Rect::empty(),
        };
        let id = f(&mut child_cx)?;
        let _ = self.children.push(id);
        Ok(self)
    }

    /// Appends an existing widget ID as a child.
    pub fn child_widget(mut self, id: WidgetId) -> Self {
        let _ = self.children.push(id);
        self
    }

    /// Fluent child helper: directly appends a themed label without an inner closure.
    pub fn child_label(self, text: &'a str) -> Result<Self, GuiError> {
        self.child(|c| c.label(text))
    }

    /// Fluent child helper: directly appends a themed button without an inner closure.
    pub fn child_button(self, text: &'a str) -> Result<Self, GuiError> {
        self.child(|c| c.button(text))
    }

    /// Fluent child helper: directly nests a column container.
    pub fn child_column<F>(self, builder: F) -> Result<Self, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        self.child(|c| c.column(builder))
    }

    /// Fluent child helper: directly nests a row container.
    pub fn child_row<F>(self, builder: F) -> Result<Self, GuiError>
    where
        F: FnOnce(FlexBuilder<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        self.child(|c| c.row(builder))
    }

    /// Conditionally appends a child element when `condition` is true.
    pub fn when_child<F>(self, condition: bool, f: F) -> Result<Self, GuiError>
    where
        F: FnOnce(&mut ViewContext<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        if condition { self.child(f) } else { Ok(self) }
    }

    /// Builds and positions all child elements within the layout container.
    pub fn build(self) -> Result<WidgetId, GuiError> {
        let panel_id = if let Some(style) = self.style_override {
            self.ctx.add_panel(self.bounds, style)?
        } else {
            self.ctx.add_themed_panel(self.bounds)?
        };
        let count = self.children.len();
        if count == 0 {
            return Ok(panel_id);
        }

        let mut rects = [Rect::empty(); 16];
        self.layout.arrange(self.bounds, count, &mut rects[..count]);

        for (i, &child_id) in self.children.iter().enumerate() {
            let child_rect = rects[i];
            self.ctx.set_widget_rect(child_id, child_rect)?;
            self.ctx.add_child(panel_id, child_id)?;
        }

        Ok(panel_id)
    }
}

impl<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    crate::geometry::FluentBuilder for FlexBuilder<'a, 'ctx, NODES, EVENTS, DIRTY>
{
}

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    /// Builds a declarative pure-Rust view into the GUI context.
    pub fn build_view<F>(&mut self, builder: F) -> Result<WidgetId, GuiError>
    where
        F: FnOnce(&mut ViewContext<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        let viewport = self.viewport();
        let mut cx = ViewContext::new(self, viewport);
        builder(&mut cx)
    }

    /// Renders a component implementing the [`Render`] trait.
    pub fn render_view<V: Render>(&mut self, view: &V) -> Result<WidgetId, GuiError> {
        let viewport = self.viewport();
        let mut cx = ViewContext::new(self, viewport);
        view.render(&mut cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ClimateCard {
        title: &'static str,
        is_eco: bool,
    }

    impl Render for ClimateCard {
        fn render<'a, 'ctx, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
            &self,
            cx: &mut ViewContext<'a, 'ctx, NODES, EVENTS, DIRTY>,
        ) -> Result<WidgetId, GuiError> {
            cx.column(|col| {
                col.padding((8, 12))
                    .gap(6)
                    .child(|c| c.label(self.title))?
                    .child(|c| c.button("SET AUTO"))?
                    .when_child(self.is_eco, |c| c.label("ECO ACTIVE"))?
                    .build()
            })
        }
    }

    #[test]
    fn test_declarative_view_rendering() {
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(Rect::new(0, 0, 320, 240));
        let view = ClimateCard {
            title: "Living Room",
            is_eco: true,
        };

        let root = ctx.render_view(&view).unwrap();
        assert_eq!(ctx.widgets().len(), 4); // panel container + 3 children
        assert!(ctx.node(root).is_some());
    }

    #[test]
    fn test_view_row_panel_and_flex_when_child_false() {
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(Rect::new(0, 0, 320, 240));
        let root = ctx
            .build_view(|cx| {
                cx.row(|row| {
                    row.child(|c| c.styled_label("A", crate::style::Style::label()))?
                        .child(|c| {
                            c.panel(|panel| panel.styled_label("P", crate::style::Style::label()))
                        })?
                        .when_child(false, |c| c.label("never"))?
                        .build()
                })
            })
            .unwrap();
        assert!(ctx.node(root).is_some());

        let root2 = ctx
            .build_view(|cx| {
                cx.column(|col| {
                    col.cross_align(Align::Center)
                        .justify(JustifyContent::SpaceBetween)
                        .child(|c| c.button("B"))?
                        .build()
                })
            })
            .unwrap();
        assert!(ctx.node(root2).is_some());
    }

    #[cfg(feature = "rich-widgets")]
    #[test]
    fn test_view_rich_widget_builders() {
        let mut ctx: GuiContext<16, 16, 16> = GuiContext::new(Rect::new(0, 0, 160, 120));
        let root = ctx
            .build_view(|cx| {
                cx.column(|col| {
                    col.child(|c| c.toggle("T", true))?
                        .child(|c| c.slider(0.5, 0.0, 1.0))?
                        .child(|c| c.progress_bar(0.7))?
                        .build()
                })
            })
            .unwrap();
        assert!(ctx.node(root).is_some());
    }

    #[test]
    fn test_guillotine_style_declarative_builder() {
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(Rect::new(0, 0, 320, 240));
        let root = ctx
            .build_view(|cx| {
                cx.div(|div| {
                    div.size(320, 172)
                        .padding(12)
                        .gap(8)
                        .bg(Rgb565::new(2, 4, 6))
                        .border(1, Rgb565::new(7, 47, 25))
                        .corner_radius(4)
                        .align_items(Align::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .child_label("GUILLOTINE")?
                        .child_button("READY")?
                        .child_row(|row| {
                            row.gap(4)
                                .child_label("STATUS")?
                                .child_button("OK")?
                                .build()
                        })?
                        .build()
                })
            })
            .unwrap();

        assert!(ctx.node(root).is_some());
        // Root panel + label + button + inner row panel + inner label + inner button = 6 widgets
        assert_eq!(ctx.widgets().len(), 6);
    }
}
