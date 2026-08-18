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
  assets/
    logo.png
    warning.bmp
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

## Raster image assets

Use **File → Import Image Asset…** to copy a PNG, JPEG, or BMP into the
project's `assets/` directory and add it to the active screen. Image paths are
project-relative:

```kdl
image id="logo" src="assets/logo.png" fit="center" mode="color" col=0 row=0
image id="warning" src="assets/warning.bmp" fit="stretch" mode="mask"
      tint="#00FFFF" col=1 row=0
```

`mode="color"` converts full-color and alpha images to RGB565.
`mode="mask"` treats dark source pixels as ink and supports a semantic or hex
`tint`; this is useful for recolorable 1-bit icons. Studio uses the same RGB565
conversion as `include_gui!`, which embeds the converted pixels as static
`no_std` data—firmware does not carry a PNG/JPEG decoder.

A checked-in example lives at
[`crates/embedded-gui-studio/examples/ssd1357-demo`](../crates/embedded-gui-studio/examples/ssd1357-demo).

## Carousels

A `carousel` is a wrap-around list drawn as one widget, so its slot falloff,
edge fade, and chrome masks are identical in the preview and on the panel:

```kdl
carousel id="items" selected=2 item_step=16 visible=7 shift=0
         mask_top=14 mask_bottom=12 fade=true indicator=true pulse=96
         style="body" col=0 row=1 {
    option "DEFAULT"
    option "STEALTH OFF"
    option "BRIGHTNESS"
}
```

- `item_step` is the vertical pitch between rows and `visible` the number of
  rows drawn, centered on `selected`. Rows past the ends wrap around.
- Rows dim with distance from the selection, so depth reads without motion.
- `shift` is the in-flight scroll offset in pixels. Animate it from firmware
  (`CarouselSpec::shift`) between steps and snap it back to `0` when the
  selection changes; a 4px shift moves every row exactly 4px.
- `mask_top` / `mask_bottom` repaint the backdrop immediately above and below
  the widget, so rows overhanging the rect slide *behind* a header and footer.
  Declare that chrome after the carousel so it draws on top.
- `indicator` flanks the selected row with accent bars; `pulse` (0–255) scales
  their color, which is where a breathing highlight comes from.

## Imported fonts

Declare a BDF font on the screen (**File → Import Font (BDF)…**) and reference
it by name. `chars` limits which glyphs are embedded — a shot counter only
needs digits, and dropping the rest saves flash:

```kdl
screen id="Counter" width=96 height=64 {
    font id="sevenseg" src="assets/fonts/sevenseg30.bdf" chars="0123456789"
    grid cols="1fr" rows="1fr" {
        label id="count" text="042" font="sevenseg" col=0 row=0
    }
}
```

Labels and carousels accept `font="…"`. Studio parses the BDF at edit time and
`include_gui!` bakes the same glyph bitmaps into a `BitmapFont` static.

The cell is cropped to the glyphs that were embedded rather than to the font's
declared bounding box: display faces reserve room for accents and descenders no
digit uses, and on a 96px panel that padding decides whether two digits fit.

## Composite icons

An `icon` stacks 1-bit layers at fixed offsets. Each part can be toggled and
tinted on its own, which is how one widget shows compound state (a magazine
seated or not, a bolt only while charging):

```kdl
icon id="battery" scale=1 align="center" tint="success" threshold=128 col=1 row=1 {
    part src="assets/icons/batt_shell.bmp" x=0 y=0
    part src="assets/icons/batt_fill.bmp" x=0 y=0
    part src="assets/icons/batt_bolt.bmp" x=0 y=0 visible=false tint="accent"
}
```

Hidden parts leave their pixels untouched rather than punching a hole, and the
icon's bounds cover every part, so toggling one never shifts the others.
`threshold` sets which source luminance becomes ink; add `invert=true` for
light-on-dark art. `scale` is a nearest-neighbour upscale that keeps pixel art
crisp.

## 3D meshes

A `mesh` node renders a Wavefront OBJ or an STL (binary or ASCII) through
`embedded-3dgfx` inside its cell. The model is centered and normalized to unit
radius on import, so `scale` means the same thing for any source file, and STL's
repeated corners are welded back into shared vertices:

```kdl
mesh id="logo" src="assets/meshes/gem.obj" shading="lit" color="accent"
     scale=1.0 roll=0.4 pitch=0.7 yaw=0.0 camera_distance=3.5 fov=1.2
     col=0 row=2
```

`shading` is `solid`, `lit`, `lines`, or `points`. Because the rasterizer needs
a Z-buffer the application owns, the generated code exposes the panel as a
function instead of a widget; the node still reserves its rect:

```rust
let rect = gui.absolute_rect(app.widgets.logo).unwrap();
let mut zbuffer = [0u32; 96 * 14];
render_mesh_panel(&mut display, rect, &counter::logo_mesh_panel(), &mut zbuffer)?;
```

Spin it by overwriting `panel.attitude` each frame. Firmware needs
embedded-gui's `embedded-3dgfx` feature for this node.

A worked example of all four features is
[`examples/basics/kdl_advanced_widgets_showcase.rs`](../crates/embedded-gui/examples/basics/kdl_advanced_widgets_showcase.rs),
which compiles the demo project's `counter.kdl` and `menu.kdl`.

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

Carousels, imported fonts, composite icons, and meshes are compiled the same
way: `include_gui!` bakes the font glyphs, 1-bit icon layers, and mesh geometry
into static `no_std` data next to the generated builder.
