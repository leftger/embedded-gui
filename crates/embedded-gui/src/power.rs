//! Power management, inactivity timeouts, and dynamic refresh rate throttling.
//!
//! Designed for battery-operated embedded devices (wearables, field sensors, IoT thermostats)
//! to minimize MCU cycles and display backlight consumption.

/// The display power and activity state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerState {
    /// Screen is fully lit and operating at standard interactive framerate.
    #[default]
    Active,
    /// Screen backlight is dimmed after initial inactivity; refresh rate is throttled.
    Dimmed,
    /// Backlight is off; display is in standby awaiting input or interrupt.
    Standby,
    /// System is shut down or in deep sleep.
    Off,
}

/// Transition event emitted when power state changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    /// No change in power state.
    None,
    /// State transitioned from previous to next.
    StateChanged { from: PowerState, to: PowerState },
    /// Woke from a lower power state due to user activity.
    Woken { from: PowerState },
}

/// Inactivity timeout and brightness configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerConfig {
    /// Time in milliseconds of inactivity before dimming the display.
    pub dim_timeout_ms: u32,
    /// Time in milliseconds of inactivity before entering standby.
    pub standby_timeout_ms: u32,
    /// Backlight PWM level in `[0, 255]` while in [`PowerState::Active`].
    pub active_brightness: u8,
    /// Backlight PWM level in `[0, 255]` while in [`PowerState::Dimmed`].
    pub dim_brightness: u8,
    /// Target frame rate (FPS) while active.
    pub active_fps: u8,
    /// Target frame rate (FPS) while dimmed.
    pub dim_fps: u8,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            dim_timeout_ms: 30_000,     // 30 seconds
            standby_timeout_ms: 60_000, // 60 seconds
            active_brightness: 255,     // 100%
            dim_brightness: 64,         // 25%
            active_fps: 60,
            dim_fps: 10,
        }
    }
}

/// State machine managing display power, inactivity timers, and refresh throttling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerManager {
    state: PowerState,
    inactivity_ms: u32,
    config: PowerConfig,
    wake_on_input: bool,
}

impl PowerManager {
    /// Creates a new power manager with the given configuration.
    pub const fn new(config: PowerConfig) -> Self {
        Self {
            state: PowerState::Active,
            inactivity_ms: 0,
            config,
            wake_on_input: true,
        }
    }

    /// Whether input events automatically wake the device from `Dimmed` or `Standby`.
    pub fn set_wake_on_input(&mut self, enable: bool) {
        self.wake_on_input = enable;
    }

    /// Current display power state.
    pub const fn state(&self) -> PowerState {
        self.state
    }

    /// Current duration of continuous user inactivity in milliseconds.
    pub const fn inactivity_ms(&self) -> u32 {
        self.inactivity_ms
    }

    /// Active configuration parameters.
    pub const fn config(&self) -> &PowerConfig {
        &self.config
    }

    /// Mutable reference to configuration parameters.
    pub fn config_mut(&mut self) -> &mut PowerConfig {
        &mut self.config
    }

    /// Advances the inactivity timer by `delta_ms` and checks for state transitions.
    pub fn tick(&mut self, delta_ms: u32) -> PowerEvent {
        if self.state == PowerState::Off || self.state == PowerState::Standby {
            return PowerEvent::None;
        }

        self.inactivity_ms = self.inactivity_ms.saturating_add(delta_ms);

        let prev = self.state;
        if self.inactivity_ms >= self.config.standby_timeout_ms {
            self.state = PowerState::Standby;
        } else if self.inactivity_ms >= self.config.dim_timeout_ms {
            self.state = PowerState::Dimmed;
        }

        if self.state != prev {
            PowerEvent::StateChanged {
                from: prev,
                to: self.state,
            }
        } else {
            PowerEvent::None
        }
    }

    /// Notifies the power manager of user interaction (touch, button, encoder).
    ///
    /// Resets the inactivity timer. If the display was dimmed or in standby,
    /// wakes the system back to [`PowerState::Active`].
    pub fn notify_activity(&mut self) -> PowerEvent {
        self.inactivity_ms = 0;
        if self.wake_on_input
            && (self.state == PowerState::Dimmed || self.state == PowerState::Standby)
        {
            let prev = self.state;
            self.state = PowerState::Active;
            PowerEvent::Woken { from: prev }
        } else {
            PowerEvent::None
        }
    }

    /// Forcibly transitions into a specific power state.
    pub fn force_state(&mut self, state: PowerState) -> PowerEvent {
        let prev = self.state;
        self.state = state;
        if state == PowerState::Active {
            self.inactivity_ms = 0;
        }
        if self.state != prev {
            PowerEvent::StateChanged {
                from: prev,
                to: self.state,
            }
        } else {
            PowerEvent::None
        }
    }

    /// Returns recommended display backlight brightness in `[0, 255]`.
    pub const fn backlight_brightness(&self) -> u8 {
        match self.state {
            PowerState::Active => self.config.active_brightness,
            PowerState::Dimmed => self.config.dim_brightness,
            PowerState::Standby | PowerState::Off => 0,
        }
    }

    /// Returns recommended target frame rate (frames per second).
    pub const fn target_fps(&self) -> u8 {
        match self.state {
            PowerState::Active => self.config.active_fps,
            PowerState::Dimmed => self.config.dim_fps,
            PowerState::Standby | PowerState::Off => 0,
        }
    }

    /// Returns recommended frame interval in milliseconds, or `None` if rendering should halt.
    pub const fn frame_interval_ms(&self) -> Option<u32> {
        match self.state {
            PowerState::Active => 1000u32.checked_div(self.config.active_fps as u32),
            PowerState::Dimmed => 1000u32.checked_div(self.config.dim_fps as u32),
            PowerState::Standby | PowerState::Off => None,
        }
    }

    /// Whether the display should render new frames in the current state.
    pub const fn should_render(&self) -> bool {
        matches!(self.state, PowerState::Active | PowerState::Dimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_manager_lifecycle() {
        let config = PowerConfig {
            dim_timeout_ms: 1000,
            standby_timeout_ms: 3000,
            active_brightness: 255,
            dim_brightness: 50,
            active_fps: 60,
            dim_fps: 10,
        };

        let mut pm = PowerManager::new(config);
        assert_eq!(pm.state(), PowerState::Active);
        assert_eq!(pm.backlight_brightness(), 255);
        assert_eq!(pm.target_fps(), 60);
        assert!(pm.should_render());

        // Advance 500ms -> Still active
        assert_eq!(pm.tick(500), PowerEvent::None);
        assert_eq!(pm.state(), PowerState::Active);

        // Advance another 600ms (total 1100ms) -> Dimmed
        assert_eq!(
            pm.tick(600),
            PowerEvent::StateChanged {
                from: PowerState::Active,
                to: PowerState::Dimmed,
            }
        );
        assert_eq!(pm.state(), PowerState::Dimmed);
        assert_eq!(pm.backlight_brightness(), 50);
        assert_eq!(pm.target_fps(), 10);

        // Activity wakes back to active
        assert_eq!(
            pm.notify_activity(),
            PowerEvent::Woken {
                from: PowerState::Dimmed
            }
        );
        assert_eq!(pm.state(), PowerState::Active);
        assert_eq!(pm.inactivity_ms(), 0);

        // Advance 3100ms -> Standby
        assert_eq!(
            pm.tick(3100),
            PowerEvent::StateChanged {
                from: PowerState::Active,
                to: PowerState::Standby,
            }
        );
        assert_eq!(pm.state(), PowerState::Standby);
        assert_eq!(pm.backlight_brightness(), 0);
        assert_eq!(pm.target_fps(), 0);
        assert!(!pm.should_render());

        // Wakes from Standby
        assert_eq!(
            pm.notify_activity(),
            PowerEvent::Woken {
                from: PowerState::Standby
            }
        );
        assert_eq!(pm.state(), PowerState::Active);
    }

    #[test]
    fn test_power_manager_off_config_and_wake_disabled() {
        let config = PowerConfig {
            dim_timeout_ms: 100,
            standby_timeout_ms: 200,
            active_brightness: 255,
            dim_brightness: 10,
            active_fps: 60,
            dim_fps: 15,
        };
        let mut pm = PowerManager::new(config);
        pm.set_wake_on_input(false);
        pm.tick(250);
        assert_eq!(pm.state(), PowerState::Standby);
        assert_eq!(pm.frame_interval_ms(), None);
        assert_eq!(pm.notify_activity(), PowerEvent::None);
        assert_eq!(pm.state(), PowerState::Standby);
        assert_eq!(pm.tick(1000), PowerEvent::None);

        assert_eq!(
            pm.force_state(PowerState::Off),
            PowerEvent::StateChanged {
                from: PowerState::Standby,
                to: PowerState::Off,
            }
        );
        assert_eq!(pm.tick(100), PowerEvent::None);
        assert_eq!(pm.force_state(PowerState::Off), PowerEvent::None);
        assert_eq!(pm.frame_interval_ms(), None);

        pm.force_state(PowerState::Dimmed);
        assert_eq!(pm.backlight_brightness(), 10);
        assert_eq!(pm.target_fps(), 15);
        assert_eq!(pm.frame_interval_ms(), Some(1000 / 15));
        assert!(pm.should_render());

        pm.config_mut().active_fps = 30;
        pm.force_state(PowerState::Active);
        assert_eq!(pm.config().active_fps, 30);
        assert_eq!(pm.frame_interval_ms(), Some(33));

        let default_cfg = PowerConfig::default();
        assert_eq!(default_cfg.dim_timeout_ms, 30_000);
    }
}
