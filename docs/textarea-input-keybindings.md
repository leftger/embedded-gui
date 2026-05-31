# Textarea Input Keybindings

`embedded-gui` textareas support editor-style cursor movement and editing via `InputEvent`.

## Supported Navigation Events

- `InputEvent::Left` / `InputEvent::Right`: move cursor by one character.
- `InputEvent::WordLeft` / `InputEvent::WordRight`: jump cursor by word boundary.
- `InputEvent::Home` / `InputEvent::End`: jump to wrapped-line start/end.
- `InputEvent::SelectLeft` / `InputEvent::SelectRight`: expand selection by one character.
- `InputEvent::SelectWordLeft` / `InputEvent::SelectWordRight`: expand selection by word.
- `InputEvent::SelectHome` / `InputEvent::SelectEnd`: expand selection to wrapped-line boundary.
- `InputEvent::Back`: backspace (delete char before cursor or current selection).
- `InputEvent::Undo` / `InputEvent::Redo`: navigate fixed-capacity textarea edit history.

## Selection + Edit Semantics

- Typing with an active selection replaces the selected range.
- `Backspace` or `DeleteForward` with a selection removes the selected range first.
- Selection clears automatically after a mutating edit.
- `read_only` textareas ignore edit mutations.
- `single_line` textareas reject newline insertion.

## Typical Event Loop Mapping

```rust
use embedded_gui::prelude::*;

fn route_input(gui: &mut GuiContext<32, 64, 32>, nav: NavKey) -> Result<(), GuiError> {
    let event = match nav {
        NavKey::Left => InputEvent::Left,
        NavKey::Right => InputEvent::Right,
        NavKey::WordLeft => InputEvent::WordLeft,
        NavKey::WordRight => InputEvent::WordRight,
        NavKey::Home => InputEvent::Home,
        NavKey::End => InputEvent::End,
        NavKey::SelectWordRight => InputEvent::SelectWordRight,
        NavKey::Undo => InputEvent::Undo,
        NavKey::Redo => InputEvent::Redo,
        NavKey::Backspace => InputEvent::Back,
        NavKey::Enter => InputEvent::Select,
    };
    gui.handle_input(event)
}
```

Use `gui.tick_input(dt_ms)` in your frame loop to keep long-press and cursor-blink state updated.
