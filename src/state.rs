#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    pub selected: usize,
    pub offset: usize,
    pub visible_rows: usize,
}

impl ListState {
    pub const fn new(selected: usize, offset: usize, visible_rows: usize) -> Self {
        Self {
            selected,
            offset,
            visible_rows,
        }
    }

    pub fn set_selected(&mut self, selected: usize, len: usize) -> bool {
        let next = selected.min(len.saturating_sub(1));
        let changed = next != self.selected;
        self.selected = next;
        self.keep_selected_visible();
        changed
    }

    pub fn next(&mut self, len: usize) -> bool {
        self.bump(len, 1)
    }

    pub fn previous(&mut self, len: usize) -> bool {
        self.bump(len, -1)
    }

    pub fn bump(&mut self, len: usize, delta: i8) -> bool {
        if len == 0 {
            return false;
        }
        let next = if delta >= 0 {
            (self.selected + 1) % len
        } else if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        self.set_selected(next, len)
    }

    pub fn keep_selected_visible(&mut self) {
        let rows = self.visible_rows.max(1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset.saturating_add(rows) {
            self.offset = self.selected.saturating_add(1).saturating_sub(rows);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TabsState {
    pub selected: usize,
}

impl TabsState {
    pub const fn new(selected: usize) -> Self {
        Self { selected }
    }

    pub fn set_selected(&mut self, selected: usize, len: usize) -> bool {
        let next = selected.min(len.saturating_sub(1));
        let changed = next != self.selected;
        self.selected = next;
        changed
    }

    pub fn next(&mut self, len: usize) -> bool {
        self.bump(len, 1)
    }

    pub fn previous(&mut self, len: usize) -> bool {
        self.bump(len, -1)
    }

    pub fn bump(&mut self, len: usize, delta: i8) -> bool {
        if len == 0 {
            return false;
        }
        let next = if delta >= 0 {
            (self.selected + 1) % len
        } else if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        self.set_selected(next, len)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollState {
    pub offset_y: i32,
    pub content_h: u32,
}

impl ScrollState {
    pub const fn new(offset_y: i32, content_h: u32) -> Self {
        Self {
            offset_y,
            content_h,
        }
    }

    pub fn set_offset(&mut self, offset_y: i32) -> bool {
        let next = offset_y.clamp(0, self.content_h as i32);
        let changed = next != self.offset_y;
        self.offset_y = next;
        changed
    }

    pub fn scroll_by(&mut self, delta_y: i32) -> bool {
        self.set_offset(self.offset_y.saturating_add(delta_y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderState {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl SliderState {
    pub const fn new(value: f32, min: f32, max: f32) -> Self {
        Self { value, min, max }
    }

    pub fn set_value(&mut self, value: f32) -> bool {
        let next = value.clamp(self.min.min(self.max), self.min.max(self.max));
        let changed = (next - self.value).abs() > f32::EPSILON;
        self.value = next;
        changed
    }

    pub fn step_by(&mut self, direction: f32) -> bool {
        let step = ((self.max - self.min).abs() / 20.0).max(0.01);
        self.set_value(self.value + step * direction)
    }
}
