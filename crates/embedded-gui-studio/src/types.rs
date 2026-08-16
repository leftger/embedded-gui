//! Studio types and interactive drag state definitions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioTab {
    VisualPreview,
    RustCodegen,
    AstHierarchy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveDrag {
    None,
    ResizeColDivider { col_idx: usize },
    ResizeRowDivider { row_idx: usize },
    MoveWidget { widget_idx: usize },
    ResizeWidgetSpan { widget_idx: usize },
}
