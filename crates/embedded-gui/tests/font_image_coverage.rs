//! Coverage tests for font IDs/glyph helpers and image tiling/atlas/decoding.

use embedded_gui::font::{FontId, get_vector_glyph, glyph_rows};
use embedded_gui::geometry::Rect;
use embedded_gui::image::{
    ImageAtlas, ImageAtlasEntry, ImageFit, ImageRef, ReelFrame, ReelPlayer, SpriteSheet, TileMode,
    TileRef,
};

#[test]
fn font_ids_and_glyph_rows() {
    assert_eq!(FontId::Tiny3x5.advance(), 4);
    assert_eq!(FontId::Tiny3x5.line_height(), 6);
    assert_eq!(FontId::Medium4x7.advance(), 5);
    assert_eq!(FontId::Scaled6x10.advance(), 7);
    assert_eq!(FontId::Vector(2).advance(), 16);
    assert!(!glyph_rows(FontId::Tiny3x5, 'A').is_empty());
}

#[test]
fn vector_glyph_catalog_covers_printable_ascii() {
    let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-+.:/";
    for ch in chars.chars() {
        assert!(!get_vector_glyph(ch).is_empty());
    }
    let _ = get_vector_glyph(' ');
    for ch in (' '..='~').filter(|c| !chars.contains(*c)) {
        let _ = get_vector_glyph(ch);
    }
    for ch in "Hello, GUI! 12345".chars() {
        let _ = glyph_rows(FontId::Tiny3x5, ch);
    }
    let _ = glyph_rows(FontId::Tiny3x5, 'é');
}

#[test]
fn image_ref_bounds_and_tile_pixel_modes() {
    static PIXELS: [u16; 4] = [
        0xF800, // red
        0x07E0, // green
        0x001F, // blue
        0xFFFF, // white
    ];
    let image = ImageRef::new(2, 2, &PIXELS);
    assert_eq!(
        image.bounds_at(Rect::new(0, 0, 10, 10), ImageFit::Stretch),
        Rect::new(0, 0, 10, 10)
    );
    assert_eq!(
        image.bounds_at(Rect::new(0, 0, 10, 10), ImageFit::Center).w,
        2
    );
    assert_eq!(
        image.bounds_at(Rect::new(0, 0, 10, 10), ImageFit::Center).h,
        2
    );

    let none = TileRef::new(2, 2, &PIXELS, TileMode::None);
    assert!(none.get_pixel(-1, 0).is_none());
    assert!(none.get_pixel(0, 0).is_some());

    let repeat = TileRef::new(2, 2, &PIXELS, TileMode::Repeat);
    assert_eq!(repeat.get_pixel(2, 0), repeat.get_pixel(0, 0));
    assert_eq!(repeat.get_pixel(0, 2), repeat.get_pixel(0, 0));

    let mirror = TileRef::from_image(image, TileMode::Mirror);
    assert_eq!(mirror.get_pixel(1, 0), mirror.get_pixel(2, 0));
}

#[test]
fn sprite_sheet_reel_player_and_atlas() {
    static PIXELS: [u16; 16] = [0; 16];
    let image = ImageRef::new(4, 4, &PIXELS);
    let sheet = SpriteSheet::new(image, 2, 2);
    assert_eq!(sheet.sprite_rect(0), Rect::new(0, 0, 2, 2));
    assert_eq!(sheet.sprite_rect(1), Rect::new(2, 0, 2, 2));
    assert_eq!(sheet.sprite_rect(2), Rect::new(0, 2, 2, 2));

    static FRAMES: [ReelFrame; 2] = [
        ReelFrame {
            sprite_index: 0,
            duration_ms: 10,
        },
        ReelFrame {
            sprite_index: 1,
            duration_ms: 10,
        },
    ];
    let mut reel = ReelPlayer::new(sheet, &FRAMES, false);
    assert_eq!(reel.current_sprite_rect(), Some(Rect::new(0, 0, 2, 2)));
    reel.tick(5);
    assert!(!reel.is_finished());
    reel.tick(10);
    assert!(!reel.is_finished());
    assert_eq!(reel.current_sprite_rect(), Some(Rect::new(2, 0, 2, 2)));
    reel.tick(10);
    assert!(reel.is_finished());
    reel.restart();
    assert!(!reel.is_finished());

    static ENTRIES: [ImageAtlasEntry; 1] = [ImageAtlasEntry {
        id: 7,
        rect: Rect::new(1, 2, 3, 4),
    }];
    let atlas = ImageAtlas::new(image, &ENTRIES);
    assert_eq!(atlas.rect_for(7), Some(Rect::new(1, 2, 3, 4)));
    assert_eq!(atlas.rect_for(8), None);
}

#[cfg(all(feature = "std", feature = "image-decode"))]
#[test]
fn ppm_image_decoder() {
    use embedded_gui::image::{
        BasicImageDecoder, EncodedImageFormat, ImageDecodeError, decode_image_auto,
        decode_image_with, decode_ppm_ascii,
    };

    let ppm = "P3\n2 1\n255\n255 0 0  0 255 0\n";
    let mut out = heapless::Vec::<u16, 8>::new();
    let (w, h) = decode_ppm_ascii(ppm, &mut out).unwrap();
    assert_eq!((w, h), (2, 1));
    assert_eq!(out.len(), 2);
    assert_eq!(
        decode_image_with(
            &BasicImageDecoder,
            EncodedImageFormat::PpmAscii,
            ppm,
            &mut out
        ),
        Ok((2, 1))
    );
    assert_eq!(decode_image_auto(ppm, &mut out), Ok((2, 1)));
    assert_eq!(
        decode_image_auto("not ppm", &mut out),
        Err(ImageDecodeError::Unsupported)
    );
    assert_eq!(
        decode_ppm_ascii("P9 1 1 1 0", &mut out),
        Err(ImageDecodeError::InvalidHeader)
    );
}
