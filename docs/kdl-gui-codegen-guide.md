# Declarative KDL GUI Markup & Codegen Guide

`embedded-gui` features a compile-time declarative markup system powered by **KDL** (a human-friendly, clean, structured document language) and procedural macros.

Non-technical domain experts and UI designers can define complete embedded screens in declarative markup, which compiles directly into zero-allocation, deterministic `#![no_std]` Rust code.

---

## Table of Contents
1. [Why KDL & Declarative Codegen?](#why-kdl--declarative-codegen)
2. [Quick Start](#quick-start)
3. [KDL Screen Document Structure](#kdl-screen-document-structure)
4. [2D Grid Layout Engine](#2d-grid-layout-engine)
5. [Complete Widget Catalog](#complete-widget-catalog)
6. [Vector Bézier Paths](#vector-bezier-paths)
7. [Working with the Generated Rust Code](#working-with-the-generated-rust-code)

---

## Why KDL & Declarative Codegen?

- **Zero Runtime Allocations**: Markup is compiled into fixed arrays and typed structs at build time. No parsing, heap allocation, or DOM tree traversal happens at runtime on your microcontroller.
- **Strongly-Typed Widget References**: Every widget with an `id="..."` generates a dedicated field on the screen's widget struct with a strongly-typed `WidgetId`.
- **Clean, Human-Friendly Syntax**: Unlike XML or JSON, KDL has minimal noise, clear property syntax, and structured child blocks.

---

## Quick Start

### 1. Enable the `macros` Feature
In your application's `Cargo.toml`:
```toml
[dependencies]
embedded-gui = { version = "0.2", features = ["macros", "rich-widgets"] }
```

### 2. Create a Screen File (`ui/climate_control.kdl`)
```kdl
screen id="ClimateControl" width=320 height=240 theme="dark" {
    grid cols="140px 1fr" rows="24px 1fr 40px" gap=6 padding=8 {
        status_bar id="Header" col=0 row=0 col_span=2 time="10:42"
        
        scale id="Tach" col=0 row=1 mode="radial" min=15.0 max=35.0 value=22.5 major_ticks=4
        spinbox id="TargetTemp" col=1 row=1 min=150 max=350 value=225 digits=3 decimals=1
        
        button id="ApplyBtn" col=0 row=2 text="APPLY"
        toggle id="EcoMode" col=1 row=2 label="ECO" checked=true
    }
}
```

### 3. Include and Instantiate in Rust
```rust
use embedded_gui::prelude::*;

// Embeds and compiles the KDL file at compile-time
include_gui!("ui/climate_control.kdl");

fn main() {
    let mut display_buffer = [Rgb565::BLACK; 320 * 240];
    let mut gui = GuiContext::<64, 32, 16>::new(Rect::new(0, 0, 320, 240));

    // Build the auto-generated screen
    let app = ClimateControlApp::build(&mut gui).expect("failed to build UI");

    // Access strongly-typed widget IDs directly:
    // app.widgets.tach
    // app.widgets.target_temp
    // app.widgets.apply_btn
    // app.widgets.eco_mode
}
```

Or write inline KDL with `gui_kdl!`:
```rust
gui_kdl!(r#"
screen id="InlineScreen" width=240 height=240 {
    grid cols="1fr" rows="1fr 1fr" gap=4 padding=4 {
        banner col=0 row=0 text="Hello Embedded!"
        button id="ClickMe" col=0 row=1 text="Press"
    }
}
"#);
```

---

## KDL Screen Document Structure

A screen document starts with a root `screen` node and contains a `grid` container:

```kdl
screen id="ScreenIdentifier" width=320 height=240 theme="dark" {
    grid cols="..." rows="..." gap=4 padding=6 {
        // Child widgets placed in grid cells
    }
}
```

---

## 2D Grid Layout Engine

The `grid` container uses a 2D track sizing algorithm supporting pixels (`px`), fractional space (`fr`), and automatic sizing (`auto`):

- `cols="100px 1fr 2fr"`: Three columns (100px fixed, 1 part remaining, 2 parts remaining).
- `rows="24px 1fr 40px"`: Top bar 24px, middle expands, bottom bar 40px.
- `gap=6`: Spacing between rows and columns in pixels.
- `padding=8`: Inner margin around the grid in pixels.

### Cell Placement & Spanning
- `col=0 row=0`: Placed at column 0, row 0.
- `col=0 row=0 col_span=2 row_span=1`: Spans across 2 columns.

---

## Complete Widget Catalog

### 1. Interactive Controls

#### Button
```kdl
button id="SaveBtn" col=0 row=0 text="SAVE" on_click="handle_save"
```

#### Toggle Switch
```kdl
toggle id="WifiTog" col=0 row=1 label="Wi-Fi" checked=true
```

#### Checkbox
```kdl
checkbox id="SyncChk" col=0 row=2 label="Auto-Sync" checked=false
```

#### Slider
```kdl
slider id="VolumeSld" col=0 row=3 min=0 max=100 value=65
```

#### Spinbox (Precision Numeric Stepper)
```kdl
spinbox id="Voltage" col=0 row=4 min=0 max=5000 value=3300 digits=4 decimals=3
```

#### Dropdown Menu
```kdl
dropdown id="ModeSelect" col=0 row=5 selected=0 {
    option "Standard"
    option "Sport"
    option "Comfort"
}
```

#### Roller (Scrollable Wheel)
```kdl
roller id="MonthPicker" col=0 row=6 selected=4 {
    option "Jan"
    option "Feb"
    option "Mar"
    option "Apr"
    option "May"
}
```

---

### 2. Metrics, Gauges & Data Displays

#### Radial & Linear Scales
```kdl
// Radial Gauge
scale id="Speedo" col=0 row=0 mode="radial" min=0.0 max=180.0 value=95.0 major_ticks=6 minor_ticks=2

// Linear Horizontal Scale
scale id="Fuel" col=1 row=0 mode="linear_horizontal" min=0.0 max=100.0 value=75.0
```

#### Sweeping Arc
```kdl
sweeping_arc id="ActivityRing" col=0 row=1 start_angle=0 end_angle=270
```

#### Progress Bar
```kdl
progress_bar id="DownloadProg" col=0 row=2 value=0.85
```

#### Busy Wheel (Spinner)
```kdl
busy_wheel id="Spinner" col=0 row=3 active=true
```

#### Plotter / Chart
```kdl
plotter id="Telemetry" col=0 row=4 mode="line" // or "bar"
```

#### Table
```kdl
table id="MetricsTable" col=0 row=5 {
    header "Metric" "Value" "Status"
    row "CPU" "42%" "OK"
    row "RAM" "180KB" "OK"
    row "TEMP" "38°C" "NOMINAL"
}
```

---

### 3. Wearable & System Cards

#### Status Bar
```kdl
status_bar id="SystemStatus" col=0 row=0 col_span=2 time="10:42"
```

#### Time Picker
```kdl
time_picker id="Alarm" col=0 row=1 hour=7 minute=30 is_12h=true is_pm=false
```

#### Number Picker
```kdl
number_picker id="HeartRate" col=0 row=2 min=40 max=220 value=135 unit="BPM"
```

#### Alert & Confirmation Dialog
```kdl
dialog id="ConfirmDialog" col=0 row=3 title="Warning" message="Low Battery (12%)" type="warning"
```

#### Content & Crumbs Indicators
```kdl
content_indicator id="PageDots" col=0 row=4 count=4 active=1
crumbs id="StepNav" col=1 row=4 count=5 active=2
```

---

## Vector Bézier Paths

You can embed vector paths with Quadratic and Cubic Bézier curves directly in KDL:

```kdl
vector_path id="SmoothWave" col=0 row=0 stroke_width=2 {
    move_to 0 50
    quad_to 50 10 100 50
    cubic_to 150 90 200 10 250 50
    line_to 300 50
    close
}
```

---

## Working with the Generated Rust Code

When you invoke `include_gui!("path/to/screen.kdl")`, the macro generates:
1. `struct <ScreenId>Widgets`: Holds the `WidgetId` of every named widget.
2. `struct <ScreenId>App`: Main screen wrapper containing `pub widgets: <ScreenId>Widgets`.
3. `impl <ScreenId>App`:
   - `pub const WIDTH: u32`
   - `pub const HEIGHT: u32`
   - `pub const NODE_COUNT: usize`
   - `pub fn build(gui: &mut GuiContext) -> Result<Self, GuiError>`

### Example: Mutating State and Handling Events
```rust
let mut gui = GuiContext::<64, 32, 16>::new(Rect::new(0, 0, 320, 240));
let app = SmartDashboardApp::build(&mut gui)?;

// Update widget state dynamically:
gui.set_slider_value(app.widgets.volume_sld, 80);
gui.set_label_text(app.widgets.status_text, "Connected");

// Dispatch touch / key input:
gui.handle_touch_event(TouchEvent::Press(Point::new(160, 120)));
```
