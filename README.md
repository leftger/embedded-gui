# embedded-gui

`embedded-gui` is a small `no_std` GUI/HUD crate for `embedded-graphics` displays.

The first milestone is intentionally focused: labels, panels, buttons, progress bars,
menus, fixed-capacity storage, button/encoder navigation, and dirty-region tracking.
It is designed to render after an `embedded-3dgfx` frame, but it does not depend on
the 3D engine at runtime.

```rust
use embedded_gui::prelude::*;

let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 128, 64));
gui.add_label(Rect::new(4, 4, 80, 8), "READY", Style::label())?;
gui.add_progress_bar(Rect::new(4, 18, 80, 8), 0.6, Style::progress())?;
gui.render(&mut display)?;
```

## Animation Quickstart

`embedded-gui` includes fixed-capacity animation primitives that stay `no_std` friendly.

```rust
use embedded_gui::prelude::*;

let mut animator = WidgetAnimator::<8, 8>::new();
let progress = gui.add_progress_bar(Rect::new(4, 18, 80, 8), 0.0, Style::progress())?;
animator.animate_progress(progress, 0.0, 1.0, 600, Easing::InOutSine)?;

// In your frame loop:
animator.tick(16, &mut gui)?;
gui.render_dirty(&mut display)?;
gui.clear_dirty();
```

## Font Glyph Overrides

The build pipeline supports external glyph overrides from:

- `assets/fonts/ascii_3x5.txt`
- `assets/fonts/ascii_4x7.txt`

Format per line:

```text
key:row0,row1,row2,row3,row4
```

- `key`: a single character or `space`
- each row: 3 bits using `0`/`1` (left to right)
- blank lines and `#` comments are ignored

Example:

```text
?:111,001,010,000,010
!:010,010,010,000,010
space:000,000,000,000,000
```

Unspecified glyphs fall back to built-in defaults in `build.rs`.

### Preview helper

Use the helper script to preview glyph rows before building:

```bash
python3 scripts/preview_glyphs.py assets/fonts/ascii_3x5.txt "?!@"
```

The script prints a small ASCII visualization for each requested glyph.

## Example Screenshots

Dashboard-style UI (placeholder):

![Dashboard Placeholder](docs/screenshots/dashboard-placeholder.svg)

Mixed font text model showcase (placeholder):

![Fonts Placeholder](docs/screenshots/fonts-placeholder.svg)
