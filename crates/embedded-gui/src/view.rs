use crate::{
    GuiContext,
    context::types::GuiError,
    geometry::{EdgeInsets, Rect},
    layout::{Align, Axis, JustifyContent, LinearLayout},
    style::WidgetStyle,
    widget::WidgetId,
};
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
        }
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

    /// Configures main-axis content justification.
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.layout.justify = justify;
        self
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

    /// Conditionally appends a child element when `condition` is true.
    pub fn when_child<F>(self, condition: bool, f: F) -> Result<Self, GuiError>
    where
        F: FnOnce(&mut ViewContext<'a, '_, NODES, EVENTS, DIRTY>) -> Result<WidgetId, GuiError>,
    {
        if condition { self.child(f) } else { Ok(self) }
    }

    /// Builds and positions all child elements within the layout container.
    pub fn build(self) -> Result<WidgetId, GuiError> {
        let panel_id = self.ctx.add_themed_panel(self.bounds)?;
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
}
