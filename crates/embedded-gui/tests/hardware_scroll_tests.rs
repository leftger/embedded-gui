use embedded_gui::geometry::Rect;
use embedded_gui::render::hardware_scroll::{
    HardwareVerticalScrollTarget, HardwareVerticalScroller, MipiDcsScrollConfig, ScrollConfigError,
    St7789ScrollConfig, opcodes,
};

#[test]
fn test_mipi_scroll_config_full_screen() {
    let cfg = St7789ScrollConfig::full_screen(320);
    assert_eq!(cfg.tfa, 0);
    assert_eq!(cfg.vsa, 320);
    assert_eq!(cfg.bfa, 0);
    assert_eq!(cfg.total_lines(), 320);

    // VSCRDEF: [0x33, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00] (320 = 0x0140)
    let vscrdef = cfg.vscrdef_command();
    assert_eq!(vscrdef[0], opcodes::VSCRDEF);
    assert_eq!(vscrdef, [0x33, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00]);

    // VSCRSADD: pointer = 160 -> [0x37, 0x00, 0xA0]
    let vscrsadd = cfg.vscrsadd_command(160);
    assert_eq!(vscrsadd[0], opcodes::VSCRSADD);
    assert_eq!(vscrsadd, [0x37, 0x00, 0xA0]);

    // NORON: [0x13]
    let noron = St7789ScrollConfig::noron_command();
    assert_eq!(noron, [0x13]);
}

#[test]
fn test_mipi_scroll_config_with_fixed_areas() {
    let cfg = St7789ScrollConfig::with_fixed_areas(32, 32, 320).unwrap();
    assert_eq!(cfg.tfa, 32);
    assert_eq!(cfg.vsa, 256);
    assert_eq!(cfg.bfa, 32);
    assert_eq!(cfg.total_lines(), 320);

    let vscrdef = cfg.vscrdef_command();
    assert_eq!(vscrdef, [0x33, 0x00, 0x20, 0x01, 0x00, 0x00, 0x20]);
}

#[test]
fn test_mipi_scroll_config_invalid() {
    let res = St7789ScrollConfig::with_fixed_areas(200, 150, 320);
    assert_eq!(res, Err(ScrollConfigError::ZeroScrollArea));

    let res2 = St7789ScrollConfig::with_fixed_areas(160, 160, 320);
    assert_eq!(res2, Err(ScrollConfigError::ZeroScrollArea));
}

#[test]
fn test_scroller_downward_and_upward_delta() {
    let cfg = St7789ScrollConfig::with_fixed_areas(20, 20, 240).unwrap(); // VSA = 200
    let mut scroller = HardwareVerticalScroller::new(240, cfg);
    assert_eq!(scroller.current_vsp(), 20);

    // Scroll down by 10 pixels (content moves up, new damage exposed at bottom of VSA)
    let damage1 = scroller.scroll_by(10);
    assert_eq!(damage1.new_vsp, 30);
    assert!(!damage1.full_redraw_required);
    assert_eq!(
        damage1.exposed_damage,
        Some(Rect::new(0, 20 + 200 - 10, 240, 10))
    );

    // Scroll down by another 15 pixels
    let damage2 = scroller.scroll_by(15);
    assert_eq!(damage2.new_vsp, 45);
    assert_eq!(
        damage2.exposed_damage,
        Some(Rect::new(0, 20 + 200 - 15, 240, 15))
    );

    // Scroll up by 5 pixels (content moves down, new damage exposed at top of VSA)
    let damage3 = scroller.scroll_by(-5);
    assert_eq!(damage3.new_vsp, 40);
    assert_eq!(damage3.exposed_damage, Some(Rect::new(0, 20, 240, 5)));

    // Zero scroll: no damage
    let damage_zero = scroller.scroll_by(0);
    assert_eq!(damage_zero.new_vsp, 40);
    assert_eq!(damage_zero.exposed_damage, None);
    assert!(!damage_zero.full_redraw_required);
}

#[test]
fn test_scroller_wrapping_and_large_jumps() {
    let cfg = St7789ScrollConfig::full_screen(320);
    let mut scroller = HardwareVerticalScroller::new(240, cfg);

    // Advance near end
    scroller.scroll_by(315);
    assert_eq!(scroller.current_vsp(), 315);

    // Advance 10 pixels past end: wraps around modulo 320 to 5
    let damage_wrap = scroller.scroll_by(10);
    assert_eq!(damage_wrap.new_vsp, 5);
    assert_eq!(
        damage_wrap.exposed_damage,
        Some(Rect::new(0, 320 - 10, 240, 10))
    );

    // Delta exceeding entire VSA (e.g. 500 lines) requires full repaint
    let damage_jump = scroller.scroll_by(500);
    assert!(damage_jump.full_redraw_required);
    assert_eq!(damage_jump.exposed_damage, Some(Rect::new(0, 0, 240, 320)));

    // Reset restores pointer to TFA
    let damage_reset = scroller.reset();
    assert_eq!(damage_reset.new_vsp, 0);
    assert!(damage_reset.full_redraw_required);
}

#[test]
fn test_hardware_scroll_target_trait() {
    struct MockDisplay {
        vscrdef_received: Option<[u8; 7]>,
        vsp_received: Option<u16>,
        normal_mode: bool,
    }

    impl HardwareVerticalScrollTarget for MockDisplay {
        type Error = ();

        fn set_vertical_scroll_area(
            &mut self,
            config: &MipiDcsScrollConfig,
        ) -> Result<(), Self::Error> {
            self.vscrdef_received = Some(config.vscrdef_command());
            self.normal_mode = false;
            Ok(())
        }

        fn set_vertical_scroll_pointer(&mut self, vsp: u16) -> Result<(), Self::Error> {
            self.vsp_received = Some(vsp);
            Ok(())
        }

        fn cancel_vertical_scroll(&mut self) -> Result<(), Self::Error> {
            self.normal_mode = true;
            Ok(())
        }
    }

    let mut display = MockDisplay {
        vscrdef_received: None,
        vsp_received: None,
        normal_mode: true,
    };

    let cfg = St7789ScrollConfig::with_fixed_areas(16, 16, 240).unwrap();
    display.set_vertical_scroll_area(&cfg).unwrap();
    assert_eq!(
        display.vscrdef_received,
        Some([0x33, 0x00, 0x10, 0x00, 0xD0, 0x00, 0x10])
    );
    assert!(!display.normal_mode);

    display.set_vertical_scroll_pointer(42).unwrap();
    assert_eq!(display.vsp_received, Some(42));

    display.cancel_vertical_scroll().unwrap();
    assert!(display.normal_mode);
}
