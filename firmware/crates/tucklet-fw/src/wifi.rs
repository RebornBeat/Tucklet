//! Wi-Fi data plane. SoftAP is the universal path: the device hosts a private
//! AP with single-use credentials only while a transfer session is active, then
//! tears it down (battery + security). Wi-Fi Aware is the seamless path where
//! the chipset + certification allow.

use anyhow::Result;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, EspWifi};
use tucklet_core::session::Session;

/// Owns the Wi-Fi radio. Brought up per session, torn down when idle.
pub struct WifiData<'d> {
    wifi: EspWifi<'d>,
    up: bool,
}

impl<'d> WifiData<'d> {
    pub fn new(
        modem: Modem,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self> {
        let wifi = EspWifi::new(modem, sysloop, Some(nvs))?;
        Ok(Self { wifi, up: false })
    }

    /// Bring up the SoftAP using the session's one-time SSID/PSK. The device's
    /// HTTP server (see httpd.rs) listens at session.ip once this is up.
    pub fn start_softap(&mut self, session: &Session) -> Result<()> {
        let ap = AccessPointConfiguration {
            ssid: session
                .ssid_or_service
                .as_str()
                .try_into()
                .unwrap_or_default(),
            password: session.psk.as_str().try_into().unwrap_or_default(),
            auth_method: AuthMethod::WPA2Personal,
            channel: 6,
            max_connections: 1, // only the paired phone
            ssid_hidden: false,
            ..Default::default()
        };
        self.wifi.set_configuration(&Configuration::AccessPoint(ap))?;
        self.wifi.start()?;
        self.up = true;
        log::info!("SoftAP up: {}", session.ssid_or_service);
        Ok(())
    }

    /// Bring up the Wi-Fi Aware (NAN) data path for a session.
    ///
    /// CONFIRM: ESP32-C5 NAN support on your ESP-IDF (some ESP variants mark NAN
    /// "Won't Do"; see docs/FINAL_REVIEW.md). The ESP-IDF NAN API is
    /// `esp_wifi_nan_start` + service publish/subscribe + datapath. This path is
    /// only reachable when built `--features wifi_aware` and the platform/device
    /// are Wi-Fi Aware certified. SoftAP remains the guaranteed fallback.
    #[cfg(feature = "wifi_aware")]
    pub fn start_aware(&mut self, session: &Session) -> Result<()> {
        let _ = session;
        // esp_wifi_nan_start(); publish service `session.ssid_or_service`;
        // establish a NAN datapath; the HTTP server binds the NAN interface IP.
        anyhow::bail!("Wi-Fi Aware path pending ESP-IDF NAN confirmation on C5");
    }

    /// Tear down the radio and invalidate the network. Called on session end,
    /// idle timeout, or disconnect.
    pub fn stop(&mut self) -> Result<()> {
        if self.up {
            self.wifi.stop()?;
            self.up = false;
            log::info!("Wi-Fi data path down");
        }
        Ok(())
    }

    pub fn is_up(&self) -> bool {
        self.up
    }
}
