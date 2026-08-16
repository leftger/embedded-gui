use embedded_graphics_core::{
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
    prelude::DrawTarget,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 320;
const H: u32 = 240;

// 1. Custom 8x16 Raw Bitmap Font Array (ASCII ' ' .. '~')
// Standard 8x16 font glyph data sample (16 bytes per glyph)
static CUSTOM_8X16_GLYPHS: [u8; 16 * 4] = [
    // ' ' (Space)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // 'A' (0x41 = index 33 in space-based offset)
    0b00000000, 0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110,
    0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
    // 'B'
    0b00000000, 0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100110, 0b01100110, 0b01111100,
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
    // 'C'
    0b00000000, 0b00111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b01100110, 0b00111100,
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
];

// Create a static BitmapFont instance
static MY_BITMAP_FONT: BitmapFont =
    BitmapFont::new_8x16(32 /* first_char: space */, 8, 16, &CUSTOM_8X16_GLYPHS);

// 2. Procedural / Dynamic Trait-based Custom Font Provider
struct ProceduralNumberFont;
impl Font for ProceduralNumberFont {
    fn advance(&self) -> u32 {
        12
    }
    fn line_height(&self) -> u32 {
        18
    }
    fn draw_glyph(&self, ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
        // Draw a double-ring box frame for any character
        for x in 0..10 {
            draw_pixel(x, 0);
            draw_pixel(x, 15);
        }
        for y in 0..16 {
            draw_pixel(0, y);
            draw_pixel(9, y);
        }
        // If it's a digit or char, draw a central cross pattern
        if ch.is_alphanumeric() {
            for i in 2..8 {
                draw_pixel(i, i + 3);
                draw_pixel(9 - i, i + 3);
            }
        }
    }
}

static PROCEDURAL_FONT: ProceduralNumberFont = ProceduralNumberFont;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Custom Font Support Showcase", &settings);

    draw_showcase(&mut display);

    'running: loop {
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit
                | SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => break 'running,
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn draw_showcase<D>(display: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let mut ctx = RenderCtx::new(display, Rect::new(0, 0, W, H));
    ctx.fill_rect(Rect::new(0, 0, W, H), Rgb565::new(3, 5, 10))
        .unwrap();

    let panel = Block::styled(Style {
        background: Some(Rgb565::new(10, 18, 28)),
        gradient: Some(LinearGradient::vertical(
            Rgb565::new(12, 22, 34),
            Rgb565::new(4, 8, 14),
        )),
        font: FontId::Scaled6x10,
        foreground: Rgb565::WHITE,
        text: Rgb565::CYAN,
        accent: Rgb565::YELLOW,
        opacity: 255,
        corner_radius: 6,
        shadow: Some(Shadow::soft()),
        border: Border::one(Rgb565::new(20, 36, 50)),
        padding: EdgeInsets::all(6),
    })
    .title("CUSTOM FONT ABSTRACTION")
    .title_align(TextAlign::Center);

    panel.render(Rect::new(10, 10, 300, 220), &mut ctx).unwrap();
    let area = panel.content_area(Rect::new(10, 10, 300, 220));

    // Section 1: Built-in bitmap font
    ctx.draw_text_in(
        Rect::new(area.x, area.y + 4, area.w, 12),
        "1. Built-in Packed Font (3x5 / 4x7):",
        TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
    )
    .unwrap();

    ctx.draw_text_in(
        Rect::new(area.x + 10, area.y + 18, area.w, 14),
        "EMBEDDED-GUI FAST BITMAP",
        TextStyle::new(Rgb565::GREEN).with_font(FontId::Medium4x7),
    )
    .unwrap();

    // Section 2: Custom Raw BitmapFont (8x16)
    ctx.draw_text_in(
        Rect::new(area.x, area.y + 38, area.w, 12),
        "2. Custom Raw BitmapFont (8x16 struct):",
        TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
    )
    .unwrap();

    let custom_bitmap_font_id = FontId::from(&MY_BITMAP_FONT);
    ctx.draw_text_in(
        Rect::new(area.x + 10, area.y + 52, area.w, 18),
        "ABC",
        TextStyle::new(Rgb565::YELLOW).with_font(custom_bitmap_font_id),
    )
    .unwrap();

    // Section 3: Dynamic trait-based font provider (`impl Font`)
    ctx.draw_text_in(
        Rect::new(area.x, area.y + 76, area.w, 12),
        "3. Dynamic Trait Provider (impl Font):",
        TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
    )
    .unwrap();

    let dyn_font_id = FontId::from(&PROCEDURAL_FONT as &'static dyn Font);
    ctx.draw_text_in(
        Rect::new(area.x + 10, area.y + 92, area.w, 20),
        "1 2 3",
        TextStyle::new(Rgb565::MAGENTA).with_font(dyn_font_id),
    )
    .unwrap();

    // Section 4: embedded-graphics MonoFont interop
    ctx.draw_text_in(
        Rect::new(area.x, area.y + 120, area.w, 12),
        "4. embedded-graphics MonoFont Interop:",
        TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
    )
    .unwrap();

    #[cfg(feature = "embedded-graphics")]
    {
        use embedded_graphics::mono_font::ascii::FONT_6X10;
        let mono_font_id = FontId::from(&FONT_6X10);
        ctx.draw_text_in(
            Rect::new(area.x + 10, area.y + 136, area.w, 14),
            "FONT_6X10 Drop-In Interop!",
            TextStyle::new(Rgb565::CYAN).with_font(mono_font_id),
        )
        .unwrap();
    }
}
