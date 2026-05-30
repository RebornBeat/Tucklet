//! Power: read the MAX17048 fuel gauge over I2C for a *real* battery percent,
//! read charge state from the charger's STAT line, and own the sleep policy.

use anyhow::Result;
use esp_idf_hal::i2c::I2cDriver;

/// MAX17048 7-bit I2C address.
const MAX17048_ADDR: u8 = 0x36;
/// SOC register (state-of-charge), 1/256 % per LSB across two bytes.
const REG_SOC: u8 = 0x04;
/// VCELL register (battery voltage), 78.125 uV per LSB.
const REG_VCELL: u8 = 0x02;

pub struct PowerMonitor<'d> {
    i2c: I2cDriver<'d>,
}

impl<'d> PowerMonitor<'d> {
    pub fn new(i2c: I2cDriver<'d>) -> Self {
        Self { i2c }
    }

    /// Battery percentage 0..=100 from the gauge's SOC register.
    pub fn battery_percent(&mut self) -> Result<u8> {
        let raw = self.read_u16(REG_SOC)?;
        // SOC high byte is the integer percent; round with the low byte.
        let pct = (raw as f32) / 256.0;
        Ok(pct.round().clamp(0.0, 100.0) as u8)
    }

    /// Battery voltage in millivolts (useful for diagnostics / low-batt cutoff).
    pub fn battery_millivolts(&mut self) -> Result<u16> {
        let raw = self.read_u16(REG_VCELL)?;
        // 78.125 uV/LSB -> mV
        Ok(((raw as f32) * 78.125 / 1000.0) as u16)
    }

    fn read_u16(&mut self, reg: u8) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(MAX17048_ADDR, &[reg], &mut buf, 100)?;
        Ok(u16::from_be_bytes(buf))
    }
}

/// Charging is read from the charger's open-drain STAT pin (low while charging
/// for the MCP73831). The caller samples the GPIO and passes the level here so
/// this stays free of HAL specifics.
pub fn charging_from_stat(stat_level_low: bool) -> bool {
    stat_level_low
}

/// Sleep policy: after this many seconds with no BLE connection and no active
/// transfer, the device drops to deep sleep (radios off). Wakes on the button
/// or the periodic advertise timer.
pub const IDLE_SLEEP_SECONDS: u32 = 60;

/// Whether to trickle on battery is gated by a minimum charge (mirrors
/// tucklet_core::trickle::MIN_BATTERY_ON_BATTERY); exposed here so power policy
/// lives in one place.
pub fn may_trickle_on_battery(battery_percent: u8) -> bool {
    battery_percent >= tucklet_core::trickle::MIN_BATTERY_ON_BATTERY
}
