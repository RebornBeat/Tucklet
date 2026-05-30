//! UI: one tactile button and one WS2812 RGB LED.
//!
//! The button is overloaded by *press pattern*, decoded from raw edge timing so
//! the logic is pure and unit-testable:
//!   * short press            -> wake / confirm pairing (during a pairing window)
//!   * long hold (>= 5 s)     -> factory reset (wipe the allow-list)
//! The LED maps 1:1 to the device state so the user always knows what's
//! happening without opening the app.

use tucklet_core::state::DeviceState;

/// Decoded button gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    None,
    ShortPress,
    LongHold,
}

/// Long-hold threshold for factory reset.
pub const LONG_HOLD_MS: u32 = 5_000;
/// Presses shorter than this are debounced away.
pub const DEBOUNCE_MS: u32 = 30;

/// Decode a completed press (press->release) of `held_ms` into a gesture.
/// Pure function: the caller measures the duration from GPIO edges.
pub fn decode_press(held_ms: u32) -> ButtonEvent {
    if held_ms < DEBOUNCE_MS {
        ButtonEvent::None
    } else if held_ms >= LONG_HOLD_MS {
        ButtonEvent::LongHold
    } else {
        ButtonEvent::ShortPress
    }
}

/// An RGB color for the status LED (0..=255 per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// Whether the LED should breathe/pulse rather than hold steady.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedStyle {
    Solid,
    Breathe,
    Blink,
}

/// Map a device state to an LED appearance. Warm, calm palette to match the
/// keepsake feel (see docs/UX_SPEC.md).
pub fn led_for(state: DeviceState, charging: bool, low_battery: bool) -> (Rgb, LedStyle) {
    if low_battery {
        return (Rgb(180, 40, 20), LedStyle::Blink); // dim red blink
    }
    match state {
        // Off when asleep (LED dark) — represented as black solid.
        DeviceState::Asleep => (Rgb(0, 0, 0), LedStyle::Solid),
        // Breathing terracotta while discoverable.
        DeviceState::Advertising => (Rgb(180, 100, 78), LedStyle::Breathe),
        // Steady warm white when connected.
        DeviceState::Connected => {
            if charging {
                (Rgb(80, 160, 90), LedStyle::Solid) // gentle green while charging
            } else {
                (Rgb(150, 130, 110), LedStyle::Solid)
            }
        }
        // Pulsing while moving data.
        DeviceState::Transferring => (Rgb(200, 140, 60), LedStyle::Breathe),
    }
}

/// A tiny stateful helper to turn raw button level samples into completed-press
/// gestures, tracking press start time. `now_ms` is a monotonic millisecond
/// clock supplied by the caller (esp_timer on-device), keeping this testable.
#[derive(Debug, Default)]
pub struct ButtonTracker {
    pressed_since_ms: Option<u32>,
}

impl ButtonTracker {
    pub fn new() -> Self {
        Self { pressed_since_ms: None }
    }

    /// Feed a level sample. `is_down` = button currently pressed (active-low
    /// already normalized by the caller). Returns a gesture on release.
    pub fn sample(&mut self, is_down: bool, now_ms: u32) -> ButtonEvent {
        match (self.pressed_since_ms, is_down) {
            (None, true) => {
                self.pressed_since_ms = Some(now_ms);
                ButtonEvent::None
            }
            (Some(start), false) => {
                self.pressed_since_ms = None;
                decode_press(now_ms.saturating_sub(start))
            }
            _ => ButtonEvent::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_decoding() {
        assert_eq!(decode_press(5), ButtonEvent::None);
        assert_eq!(decode_press(200), ButtonEvent::ShortPress);
        assert_eq!(decode_press(6000), ButtonEvent::LongHold);
    }

    #[test]
    fn tracker_emits_on_release() {
        let mut t = ButtonTracker::new();
        assert_eq!(t.sample(true, 1000), ButtonEvent::None); // press
        assert_eq!(t.sample(true, 1100), ButtonEvent::None); // held
        assert_eq!(t.sample(false, 1300), ButtonEvent::ShortPress); // release @300ms
        // long hold
        assert_eq!(t.sample(true, 2000), ButtonEvent::None);
        assert_eq!(t.sample(false, 8000), ButtonEvent::LongHold);
    }

    #[test]
    fn led_low_battery_overrides() {
        let (c, s) = led_for(DeviceState::Connected, false, true);
        assert_eq!(s, LedStyle::Blink);
        assert_eq!(c, Rgb(180, 40, 20));
    }
}
