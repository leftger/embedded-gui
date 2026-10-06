# Internationalization (i18n) & Localization Guide

`embedded-gui` provides a lightweight, zero-allocation, `#![no_std]`-compatible internationalization (i18n) and localization system. It is designed to store compact string translation tables and font subsets directly in microcontroller Flash/ROM, allowing applications to switch languages dynamically at runtime without heap allocation or widget reconstruction.

---

## 1. Core Architecture & Types

The i18n architecture lives in [`embedded_gui::i18n`](../crates/embedded-gui/src/i18n.rs) and consists of four foundational primitives:

| Type | Purpose | Memory Model |
|------|---------|--------------|
| **[`LanguageId`](../crates/embedded-gui/src/i18n.rs)** | Compact language identifier (`u8`) | 1 byte |
| **[`TranslationEntry`](../crates/embedded-gui/src/i18n.rs)** | Maps a string key to localized string slices across languages | Flash/ROM slice |
| **[`TranslationTable`](../crates/embedded-gui/src/i18n.rs)** | Collection of entries with active and fallback languages | Flash/ROM table |
| **[`MessageId`](../crates/embedded-gui/src/i18n.rs)** | Numeric index for $O(1)$ direct array lookup | 2 bytes (`u16`) |

### Language Identifiers (`LanguageId`)

Standard built-in language constants include:
- `LanguageId::EN` (English, 0)
- `LanguageId::ES` (Spanish, 1)
- `LanguageId::DE` (German, 2)
- `LanguageId::FR` (French, 3)
- `LanguageId::IT` (Italian, 4)
- `LanguageId::JA` (Japanese, 5)
- `LanguageId::ZH` (Chinese, 6)

Custom languages can be created using `LanguageId::new(id)`:
```rust
use embedded_gui::prelude::*;

const LANG_PT: LanguageId = LanguageId::new(7); // Portuguese
```

---

## 2. Defining Translation Tables

Translation tables are defined as `static` arrays stored in microcontroller Flash:

```rust
use embedded_gui::prelude::*;

static TRANSLATIONS: [TranslationEntry<'static>; 3] = [
    TranslationEntry::new("btn_start", &["Start", "Iniciar", "Starten"]),
    TranslationEntry::new("btn_stop", &["Stop", "Detener", "Stoppen"]),
    TranslationEntry::new("status_ready", &["Ready", "Listo", "Bereit"]),
];

// Create a static translation table:
static TABLE: TranslationTable<'static> = TranslationTable::new(&TRANSLATIONS);
```

### Fast Binary Search Lookups (`new_sorted`)

When entry keys are sorted alphabetically, construct the table with `new_sorted`:

```rust
// Binary search lookup in O(log N) rather than linear O(N) scan:
static SORTED_TABLE: TranslationTable<'static> = TranslationTable::new_sorted(&TRANSLATIONS);
```

### $O(1)$ Indexed Message Lookups (`MessageId`)

For performance-critical code where string comparisons are prohibitive, pass a `MessageId` to index the table directly in $O(1)$ time:

```rust
const MSG_START: MessageId = MessageId(0);
const MSG_STOP: MessageId = MessageId(1);

// Zero string comparison lookup:
let text = TABLE.translate_id(MSG_START).unwrap_or("Start");
```

---

## 3. Dynamic Formatting with Interpolation (`translate_fmt`)

Localized strings frequently contain dynamic telemetry (e.g. battery level, sensor readouts). `embedded-gui` provides zero-allocation template interpolation into stack buffers:

```rust
static ENTRIES: [TranslationEntry<'static>; 1] = [
    TranslationEntry::new(
        "battery_status",
        &["Battery: {}% ({1})", "Batería: {}% ({1})"],
    ),
];

let table = TranslationTable::new(&ENTRIES);
let mut buf: heapless::String<64> = heapless::String::new();

// Formats with sequential `{}` and positional `{1}`:
let message = table.translate_fmt("battery_status", &["85", "Charging"], &mut buf).unwrap();
assert_eq!(message, "Battery: 85% (Charging)");
```

---

## 4. Integration with `GuiContext`

Register your translation table with `GuiContext`. Switching language automatically marks the viewport dirty so widgets re-render with new translations:

```rust
let mut gui = StandardGuiContext::new(Rect::new(0, 0, 320, 240));

// Register table:
gui.set_translation_table(&TABLE);

// Translate directly:
let start_label = gui.translate("btn_start"); // "Start"

// Switch language at runtime (e.g. from a settings menu):
gui.set_language(LanguageId::ES).unwrap();

// Automatically updates translated text:
let start_label_es = gui.translate("btn_start"); // "Iniciar"
```

---

## 5. Declarative KDL Codegen Integration

When authoring screens in declarative KDL (`include_gui!`), generated screens automatically provide:
- `set_language(&self, gui, lang)`
- `set_language_with_table(&self, gui, table, lang)`

Calling `app.set_language(&mut gui, LanguageId::ES)` automatically walks all text-bearing widgets (`Label`, `Button`, `Toggle`, `Checkbox`) and updates their text through the registered translation table:

```rust
use embedded_gui::prelude::*;

include_gui!("ui/settings.kdl");

fn main() {
    let mut gui = StandardGuiContext::new(Rect::new(0, 0, 320, 240));
    gui.set_translation_table(&TABLE);

    let app = SettingsApp::build(&mut gui).unwrap();

    // Switch all widgets on screen to German:
    app.set_language(&mut gui, LanguageId::DE).unwrap();
}
```

---

## 6. Multilingual & Unicode Fonts (`SparseBitmapFont`)

Embedded microcontrollers typically cannot fit entire Unicode font files in Flash. `embedded-gui` provides [`SparseBitmapFont`](../crates/embedded-gui/src/font.rs) to store only the specific Unicode characters needed by your localized text:

```rust
use embedded_gui::font::{FontId, SparseBitmapFont};

// Bitmap arrays for characters needed:
static GLYPH_A: [u8; 8] = [0x18, 0x3C, 0x66, 0x7E, 0x66, 0x66, 0x00, 0x00];
static GLYPH_ACCENT_E: [u8; 8] = [0x0C, 0x18, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00]; // é

// Sorted array of (char, glyph_bytes)
static GLYPHS: [(char, &'static [u8]); 2] = [
    ('A', &GLYPH_A),
    ('é', &GLYPH_ACCENT_E),
];

static MULTILINGUAL_FONT: SparseBitmapFont =
    SparseBitmapFont::new(8, 8, 8, 8, 1, &GLYPHS);

// Pass directly to WidgetStyle or TextStyle:
let font_id: FontId = (&MULTILINGUAL_FONT).into();
```

---

## 7. Bidirectional (BiDi) & RTL Text Shaping

For right-to-left languages (e.g. Arabic, Hebrew):
1. Set [`TextDirection::Rtl`](../crates/embedded-gui/src/text.rs) in [`ShapingConfig`](../crates/embedded-gui/src/text.rs).
2. The built-in [`BasicTextShaper`](../crates/embedded-gui/src/text.rs) automatically:
   - Reverses RTL character runs.
   - Mirrors brackets and parentheses (`(`, `)`, `[`, `]`, `{`, `}`, `<`, `>`).
   - Preserves natural left-to-right reading order for embedded number/digit runs (e.g. `123` stays `123`).
