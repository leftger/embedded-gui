//! Lightweight haptic pattern sequencer for microcontrollers and embedded GUIs.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HapticPattern {
    None,
    Click,
    DoubleClick,
    LongPress,
    Success,
    Alert,
    Custom(&'static [(u32, u8)]),
}

static CLICK_STEPS: &[(u32, u8)] = &[(30, 255)];
static DOUBLE_CLICK_STEPS: &[(u32, u8)] = &[(40, 255), (40, 0), (40, 255)];
static LONG_PRESS_STEPS: &[(u32, u8)] = &[(150, 200)];
static SUCCESS_STEPS: &[(u32, u8)] = &[(80, 150), (40, 0), (120, 255)];
static ALERT_STEPS: &[(u32, u8)] = &[(200, 255), (80, 0), (200, 255)];

impl HapticPattern {
    pub fn steps(&self) -> &'static [(u32, u8)] {
        match self {
            HapticPattern::None => &[],
            HapticPattern::Click => CLICK_STEPS,
            HapticPattern::DoubleClick => DOUBLE_CLICK_STEPS,
            HapticPattern::LongPress => LONG_PRESS_STEPS,
            HapticPattern::Success => SUCCESS_STEPS,
            HapticPattern::Alert => ALERT_STEPS,
            HapticPattern::Custom(steps) => steps,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HapticSequencer {
    steps: &'static [(u32, u8)],
    current_step: usize,
    step_elapsed_ms: u32,
    current_intensity: u8,
}

impl Default for HapticSequencer {
    fn default() -> Self {
        Self {
            steps: &[],
            current_step: 0,
            step_elapsed_ms: 0,
            current_intensity: 0,
        }
    }
}

impl HapticSequencer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play(&mut self, pattern: HapticPattern) {
        let steps = pattern.steps();
        if steps.is_empty() {
            self.steps = &[];
            self.current_step = 0;
            self.step_elapsed_ms = 0;
            self.current_intensity = 0;
        } else {
            self.steps = steps;
            self.current_step = 0;
            self.step_elapsed_ms = 0;
            self.current_intensity = steps[0].1;
        }
    }

    pub fn stop(&mut self) {
        self.play(HapticPattern::None);
    }

    pub fn tick(&mut self, dt_ms: u32) {
        if self.steps.is_empty() {
            return;
        }

        self.step_elapsed_ms = self.step_elapsed_ms.saturating_add(dt_ms);
        
        while self.current_step < self.steps.len() && self.step_elapsed_ms >= self.steps[self.current_step].0 {
            self.step_elapsed_ms -= self.steps[self.current_step].0;
            self.current_step += 1;
        }

        if self.current_step >= self.steps.len() {
            // Finished pattern
            self.steps = &[];
            self.current_step = 0;
            self.step_elapsed_ms = 0;
            self.current_intensity = 0;
        } else {
            self.current_intensity = self.steps[self.current_step].1;
        }
    }

    pub fn current_intensity(&self) -> u8 {
        self.current_intensity
    }

    pub fn is_playing(&self) -> bool {
        !self.steps.is_empty()
    }
}
