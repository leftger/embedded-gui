use super::*;

#[test]
fn test_parse_tracks() {
    let tracks = parse_tracks("140px, 1fr 2fr auto").unwrap();
    assert_eq!(
        tracks,
        vec![
            GridTrackDef::Px(140),
            GridTrackDef::Fr(1),
            GridTrackDef::Fr(2),
            GridTrackDef::Auto,
        ]
    );
}

#[test]
fn test_compile_full_widget_suite_to_rust() {
    let kdl = r#"
screen id="FullSuite" width=320 height=240 theme="dark" {
    grid cols="100px 1fr 100px" rows="24px 1fr 1fr 40px" gap=4 padding=6 {
        status_bar id="StatusBar" col=0 row=0 col_span=3 time="10:42"
        banner col=0 row=1 text="Smart Climate"
        spinbox id="Temp" col=1 row=1 min=100 max=350 value=215
        scale id="Tach" col=2 row=1 mode="radial" min=0.0 max=120.0 value=65.0
        
        dropdown id="ModeSelect" col=0 row=2 {
            option "Auto"
            option "Cool"
            option "Heat"
        }
        number_picker id="BpmPicker" col=1 row=2 min=40 max=200 value=135 unit="BPM"
        sweeping_arc id="Sweep" col=2 row=2 start_angle=0 end_angle=180
        
        button id="SaveBtn" col=0 row=3 text="SAVE"
        toggle id="EcoTog" col=1 row=3 label="ECO" checked=true
        checkbox id="SyncChk" col=2 row=3 label="SYNC" checked=false
    }
}
"#;
    let rust_code = compile_kdl_to_rust(kdl).unwrap();
    assert!(rust_code.contains("pub struct FullSuiteWidgets {"));
    assert!(rust_code.contains("pub status_bar: WidgetId,"));
    assert!(rust_code.contains("pub temp: WidgetId,"));
    assert!(rust_code.contains("pub tach: WidgetId,"));
    assert!(rust_code.contains("pub mode_select: WidgetId,"));
    assert!(rust_code.contains("pub bpm_picker: WidgetId,"));
    assert!(rust_code.contains("pub sweep: WidgetId,"));
    assert!(rust_code.contains("pub save_btn: WidgetId,"));
    assert!(rust_code.contains("pub eco_tog: WidgetId,"));
    assert!(rust_code.contains("pub sync_chk: WidgetId,"));
    assert!(rust_code.contains("pub struct FullSuiteApp {"));
    assert!(rust_code.contains("pub const NODE_COUNT: usize = 10;"));
}

#[test]
fn test_serialize_kdl_roundtrip() {
    let original_kdl = r#"screen id="Thermostat" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="36px 1fr 48px" gap=6 padding=8 {
        status_bar id="status" time="14:32" col=0 row=0 col_span=2
        scale id="temp_gauge" mode="radial" min=15.0 max=35.0 value=22.5 major_ticks=4 minor_ticks=2 col=0 row=1
        slider id="target_slider" min=10 max=40 value=23 col=1 row=1
        button id="btn_heat" text="Heat Mode" style="accent" col=0 row=2
        toggle id="power_switch" label="Power" checked=true col=1 row=2
    }
}
"#;
    let screen = parse_kdl_screen(original_kdl).unwrap();
    let serialized = serialize_kdl_screen(&screen);
    let parsed_again = parse_kdl_screen(&serialized).unwrap();
    assert_eq!(screen, parsed_again);
}

#[test]
fn roundtrips_widget_animation_and_screen_transition() {
    let kdl = r#"screen id="Motion" width=320 height=240 transition="window_push" transition_duration=420 transition_easing="moook" transition_origin="right" {
    grid cols="1fr" rows="1fr" gap=0 padding=0 {
        button id="launch" text="Launch" animation="fade_in_up" animation_trigger="screen_enter" animation_duration=500 animation_delay=80 animation_easing="out_back" animation_repeat=2 col=0 row=0
    }
}"#;
    let screen = parse_kdl_screen(kdl).unwrap();
    let transition = screen.transition.as_ref().unwrap();
    assert_eq!(transition.preset, "window_push");
    assert_eq!(transition.duration_ms, 420);

    let animation = screen.grid.children[0].0.animation.as_ref().unwrap();
    assert_eq!(animation.preset, "fade_in_up");
    assert_eq!(animation.delay_ms, 80);
    assert_eq!(animation.repeat, 2);

    let reparsed = parse_kdl_screen(&serialize_kdl_screen(&screen)).unwrap();
    assert_eq!(screen, reparsed);

    let generated = generate_rust_code(&screen);
    assert!(generated.contains("pub const TRANSITION: ScreenTransitionSpec"));
    assert!(generated.contains("ScreenTransitionEffect::PushMoook"));
    assert!(generated.contains("ScreenTransitionOrigin::Right"));
    assert!(generated.contains("start_screen_enter_animations"));
    assert!(generated.contains("AnimatedProperty::WidgetY"));
    assert!(generated.contains("Easing::OutBack"));
}

#[test]
fn roundtrips_carousel_icon_mesh_and_fonts() {
    let kdl = r#"screen id="Advanced" width=96 height=64 {
    font id="sevenseg" src="assets/fonts/sevenseg30.bdf" chars="0123456789"
    grid cols="1fr 26px" rows="12px 1fr" gap=0 padding=0 {
        label id="count" text="042" font="sevenseg" col=0 row=0
        carousel id="items" selected=1 item_step=16 visible=7 mask_top=14 mask_bottom=12 indicator=true pulse=96 style="body" col=0 row=1 {
            option "ONE"
            option "TWO"
        }
        icon id="battery" scale=2 align="top_left" tint="success" threshold=100 invert=true col=1 row=1 {
            part src="assets/icons/shell.bmp" x=0 y=0
            part src="assets/icons/bolt.bmp" x=3 y=1 visible=false tint="accent"
        }
        mesh id="gem" src="assets/meshes/gem.obj" shading="lit" color="accent" scale=1.5 roll=0.4 pitch=0.7 yaw=0 camera_distance=3.5 fov=1.2 col=1 row=0
    }
}
"#;
    let screen = parse_kdl_screen(kdl).unwrap();
    assert_eq!(screen.fonts.len(), 1);
    assert_eq!(screen.fonts[0].chars, "0123456789");

    let reparsed = parse_kdl_screen(&serialize_kdl_screen(&screen)).unwrap();
    assert_eq!(screen, reparsed);
}

#[test]
fn generates_carousel_builder_calls() {
    let kdl = r#"screen id="Menu" width=96 height=64 {
    grid cols="1fr" rows="1fr" gap=0 padding=0 {
        carousel id="items" selected=2 item_step=16 visible=7 mask_top=14 indicator=true pulse=96 col=0 row=0 {
            option "ONE"
            option "TWO"
            option "THREE"
        }
    }
}"#;
    let rust_code = compile_kdl_to_rust(kdl).unwrap();
    assert!(rust_code.contains("gui.add_carousel(cells[0], &[\"ONE\", \"TWO\", \"THREE\"], 2,"));
    assert!(rust_code.contains("item_step: 16"));
    assert!(rust_code.contains("mask_top: 14"));
    assert!(rust_code.contains("indicator: true"));
    assert!(rust_code.contains("indicator_pulse: 96"));
}

#[test]
fn embeds_fonts_icons_and_meshes_into_generated_code() {
    let kdl = r##"screen id="Advanced" width=96 height=64 {
    font id="sevenseg" src="assets/fonts/sevenseg30.bdf"
    grid cols="1fr" rows="1fr 1fr 1fr" gap=0 padding=0 {
        label id="count" text="42" font="sevenseg" col=0 row=0
        icon id="battery" scale=2 col=0 row=1 {
            part src="assets/icons/shell.bmp" x=1 y=2
        }
        mesh id="gem" src="assets/meshes/gem.obj" shading="lit" color="#00FF00" col=0 row=2
    }
}"##;
    let screen = parse_kdl_screen(kdl).unwrap();
    let assets = ProjectAssets {
        fonts: vec![FontBinaryDef {
            name: "sevenseg".into(),
            font: assets::BitmapFontData {
                width: 8,
                height: 8,
                advance: 8,
                line_height: 8,
                first_char: b'0',
                bytes_per_row: 1,
                glyphs: vec![0xFF; 16],
            },
        }],
        icons: vec![IconAssetDef {
            source: "assets/icons/shell.bmp".into(),
            bitmap: assets::MonoBitmapData {
                width: 8,
                height: 2,
                bits: vec![0b1010_1010, 0b0101_0101],
            },
        }],
        meshes: vec![MeshAssetDef {
            source: "assets/meshes/gem.obj".into(),
            mesh: assets::MeshData {
                vertices: vec![[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                faces: vec![[0, 1, 2]],
                normals: vec![[0.0, 0.0, 1.0]],
            },
        }],
        ..ProjectAssets::default()
    };

    let rust_code = generate_rust_code_with_assets(&screen, &assets);
    assert!(rust_code.contains("static __FONT_SEVENSEG: BitmapFont = BitmapFont {"));
    assert!(rust_code.contains("FontId::Bitmap(&__FONT_SEVENSEG)"));
    assert!(rust_code.contains("pub static __ICON_1_PARTS: [IconPart<'static>; 1]"));
    assert!(rust_code.contains("pub static BATTERY_PARTS: [IconPart<'static>; 1]"));
    assert!(rust_code.contains("gui.add_composite_icon(cells[1], &__ICON_1_PARTS"));
    assert!(rust_code.contains("scale: 2"));
    assert!(rust_code.contains("pub fn gem_mesh_panel() -> MeshPanel<'static>"));
    assert!(rust_code.contains("panel.shading = MeshShading::Lit;"));
    assert!(rust_code.contains("Rgb565::new(0, 63, 0)"));
}

#[test]
fn test_svg_path_d_parsing_and_codegen() {
    let svg_d = "M 10 20 L 30 40 Q 50 60 70 80 C 10 20 30 40 50 60 Z";
    let verbs = parse_svg_path_d(svg_d);
    assert_eq!(verbs.len(), 5);
    assert_eq!(verbs[0], PathVerbDef::MoveTo(10, 20));
    assert_eq!(verbs[1], PathVerbDef::LineTo(30, 40));
    assert_eq!(verbs[2], PathVerbDef::QuadTo(50, 60, 70, 80));
    assert_eq!(verbs[3], PathVerbDef::CubicTo(10, 20, 30, 40, 50, 60));
    assert_eq!(verbs[4], PathVerbDef::Close);

    let kdl = r#"screen id="VectorApp" width=320 height=240 {
    grid cols="1fr" rows="1fr" {
        path id="my_curve" d="M 0 10 C 20 0, 40 40, 60 10 Z" stroke_width=2 col=0 row=0
    }
}"#;
    let rust_code = compile_kdl_to_rust(kdl).unwrap();
    assert!(rust_code.contains("pub struct VectorAppWidgets {"));
    assert!(rust_code.contains("pub my_curve: WidgetId,"));
    assert!(rust_code.contains("let mut _path_my_curve = VectorPath::<3>::new();"));
}

#[test]
fn image_assets_round_trip_and_generate_static_rgb565() {
    let kdl = r##"screen id="Assets" width=96 height=64 {
    grid cols="1fr" rows="1fr" {
        image id="logo" src="assets/logo.png" fit="center" mode="mask" tint="#00FFFF" col=0 row=0
    }
}"##;
    let screen = parse_kdl_screen(kdl).unwrap();
    assert_eq!(
        parse_kdl_screen(&serialize_kdl_screen(&screen)).unwrap(),
        screen
    );

    let generated = generate_rust_code_with_image_assets(
        &screen,
        &[ImageAssetDef {
            source: "assets/logo.png".into(),
            width: 2,
            height: 1,
            pixels: vec![0x0000, 0xFFFF],
        }],
    );
    assert!(generated.contains("const __IMAGE_ASSET_0_PIXELS: &[u16]"));
    assert!(generated.contains("ImageRef::new(2, 1, __IMAGE_ASSET_0_PIXELS)"));
    assert!(generated.contains("ImageFit::Center"));
}

#[test]
fn test_custom_button_styling_declarative_and_named_colors() {
    let kdl = r##"screen id="HackswellScreen" width=320 height=240 {
    grid cols="1fr" rows="1fr" {
        button id="Opt1" text="Hackswell" bg="#0A1A06" pressed_bg="#19170B" disabled_bg="forestgreen" disabled_text="crimson" col=0 row=0
    }
}"##;
    let _screen = parse_kdl_screen(kdl).unwrap();
    let rust_code = compile_kdl_to_rust(kdl).unwrap();
    assert!(rust_code.contains("gui.add_button"));
    assert!(rust_code.contains("WidgetStyle::button()"));
    assert!(rust_code.contains(".with_bg(Rgb565::new(1, 6, 0))"));
    assert!(rust_code.contains(".with_pressed_bg(Rgb565::new(3, 5, 1))"));
    assert!(rust_code.contains(".with_disabled_bg(Rgb565::new(4, 34, 4))")); // forestgreen
    assert!(rust_code.contains(".with_disabled_text(Rgb565::new(27, 5, 7))")); // crimson
}

#[test]
fn default_defs_parse_tracks_errors_and_relative_svg_paths() {
    assert_eq!(GridPlacementDef::default().col_span, 1);
    let anim = WidgetAnimationDef::default();
    assert_eq!(anim.repeat, 1);
    let transition = ScreenTransitionDef::default();
    assert_eq!(transition.duration_ms, 300);

    assert!(parse_tracks("badfr").is_err());
    assert!(parse_tracks("bogus").is_err());
    assert_eq!(parse_tracks("").unwrap(), vec![GridTrackDef::Fr(1)]);
    assert_eq!(
        parse_tracks("64 1fr auto").unwrap(),
        vec![
            GridTrackDef::Px(64),
            GridTrackDef::Fr(1),
            GridTrackDef::Auto
        ]
    );

    let rel = parse_svg_path_d("m 10 10 l 5 0 h -2 v 3 c 1 1 2 2 3 0 z");
    assert!(rel.iter().any(|v| matches!(v, PathVerbDef::LineTo(15, 10))));
    assert!(rel.iter().any(|v| matches!(v, PathVerbDef::Close)));

    let with_separators = parse_svg_path_d("M0,0 5,5 Z");
    assert_eq!(with_separators.len(), 3);
}
