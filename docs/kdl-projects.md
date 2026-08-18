# KDL Projects → Firmware Workflow

This guide covers multi-screen KDL projects in **Embedded GUI Studio**, how to
preview them on hardware (including letterboxing a small OLED layout onto a
larger TFT agent), and how firmware crates consume the same files at build time
via `include_gui!`.

## Project layout

A project is a directory with a `project.kdl` manifest and one `.kdl` file per
screen:

```text
my-ui/
  project.kdl
  screens/
    status.kdl
    menu.kdl
```

Example manifest:

```kdl
project name="my-ui" panel="ssd1357" theme="dark" {
    screen id="Status" file="screens/status.kdl"
    screen id="Menu" file="screens/menu.kdl"
}
```

| Attribute | Meaning |
|-----------|---------|
| `name` | Project title shown in Studio |
| `panel` | Optional canned target slug (`ssd1357`, `ssd1306`, `esp32_s3_box`, …) |
| `width` / `height` | Used when `panel` is omitted; exact matches select a canned profile |
| `theme` | `dark`, `light`, `amber`, `emerald`, or `mono` |
| `screen` children | Tab order; each needs `id` + `file` |

Open the folder with **File → Open Project…**. Save with **File → Save Project**
(or **Save Project As…**). Single-file Open/Save still work for one-off screens.

A checked-in example lives at
[`crates/embedded-gui-studio/examples/ssd1357-demo`](../crates/embedded-gui-studio/examples/ssd1357-demo).

## Authoring for SSD1357 (96×64)

1. Set **Target** to **SSD1357 OLED (96×64 RGB565)** (or open a project with
   `panel="ssd1357"`).
2. Keep each screen's `width`/`height` at `96`/`64`.
3. Prefer dense grids (`10px` rows, small padding) — the panel is tiny.

## Preview on a larger TFT display agent

Flash a USB display agent (for example the STM32WBA65 + ILI9341 `studio_agent`)
and connect with **Live** enabled.

Studio always fits frames to the agent's reported panel size: a 96×64 screen is
**letterboxed** onto a 320×240 panel (theme background in the margins). That is
the intended desk-preview path for compact OLED layouts when the product glass
is not attached.

The **Target** remains authoritative for editing. Leave it on **SSD1357** while
designing; do not switch Target to the connected 320×240 size unless you intend
to redesign for that panel.

## Firmware consumption (`include_gui!`)

Keep the same `.kdl` files in the firmware crate (for example under `ui/`) and
compile them in:

```toml
# Cargo.toml
embedded-gui = { version = "0.2", features = ["macros", "rich-widgets", /* … */] }
```

```rust
use embedded_gui::prelude::*;

include_gui!("ui/screens/status.kdl");

fn build_ui(gui: &mut GuiContext<64, 32, 16>) -> Result<StatusApp, GuiError> {
    StatusApp::build(gui)
}
```

Workflow:

1. Edit / preview in Studio (project open/save).
2. Commit the `project.kdl` + `screens/*.kdl` tree.
3. Firmware rebuild picks up changes through `include_gui!` — no hand-copied
   layout Rust.

Navigation, sensors, and product state machines stay in firmware. KDL owns
layout and static chrome; bind live values after `App::build` using the
typed `WidgetId` fields where the runtime API supports it.

Highly custom motion (carousels, 3D logos, bespoke fonts) can remain
hand-authored Rust beside KDL screens in the same crate.
