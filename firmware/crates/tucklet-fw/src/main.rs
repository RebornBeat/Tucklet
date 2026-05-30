//! Tucklet firmware entry point (ESP32-C5, esp-idf std).
//!
//! Two-phase init:
//!   1. Bring up the system (NVS, event loop, peripherals, fuel gauge, GPIO),
//!      build the capability descriptor from the compile-time variant, load the
//!      allow-list, and start BLE advertising.
//!   2. Run the event loop: sample the button, refresh + push status, and drain
//!      the action queue (bring up / tear down the data path, factory reset,
//!      sleep). Wi-Fi and the HTTP server live here on the main thread.
//!
//! What is real here: the structure, the state machine wiring, the variant
//! capability build, the BLE/HTTP/storage/auth modules, and all the pure logic
//! (tested in tucklet-core). What needs on-hardware bring-up: the SDMMC slot
//! config for the C5, the exact esp32-nimble callback signatures for your pinned
//! version, the WS2812 RMT timing, and (if enabled) Wi-Fi Aware/NAN.

mod app;
mod auth;
mod ble;
mod board;
mod httpd;
mod power;
mod storage;
mod ui;
mod wifi;

use std::sync::{Arc, Mutex};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{PinDriver, Pull};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

use app::{AppCommand, AppShared, AppState, StatusSnapshot};
use esp32_nimble::utilities::mutex::Mutex as NimMutex;
use tucklet_core::state::{DeviceState, Event};
use tucklet_proto::DataTransport;

fn main() -> anyhow::Result<()> {
    // Required esp-idf one-time setup.
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Tucklet firmware {} starting", board::FIRMWARE_VERSION);

    // ---- Phase 1: system + peripherals -----------------------------------
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs_part = EspDefaultNvsPartition::take()?;

    // Persistent store namespace for the allow-list + device key.
    let nvs = EspNvs::new(nvs_part.clone(), "tucklet", true)?;
    let mut auth = auth::AuthStore::load(nvs);

    // Device id + keypair (mutual auth). Seed from the hardware RNG on first boot.
    let device_id = device_id_string();
    let device_pubkey = auth.device_pubkey_hex(rand32())?;
    let device_name = format!("Tucklet {}", &device_id[device_id.len().saturating_sub(4)..]);

    // I2C to the fuel gauge.
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8, // SDA — board::PIN_I2C_SDA (CONFIRM mapping)
        peripherals.pins.gpio9, // SCL — board::PIN_I2C_SCL
        &I2cConfig::new().baudrate(400.kHz().into()),
    )?;
    let mut power_mon = power::PowerMonitor::new(i2c);

    // Button (active-low, pull-up) and charger STAT (active-low).
    let mut button = PinDriver::input(peripherals.pins.gpio5)?; // board::PIN_BUTTON
    button.set_pull(Pull::Up)?;
    let chg_stat = PinDriver::input(peripherals.pins.gpio7)?; // board::PIN_CHG_STAT

    // SD bus ownership select: low = radio owns (wireless), high = bridge owns.
    let mut sd_sel = PinDriver::output(peripherals.pins.gpio21)?; // board::PIN_SD_SEL
    sd_sel.set_low()?; // default: radio owns the card

    // Mount the card (best effort; app shows "no card" if absent).
    let card_ok = storage::mount().is_ok() && storage::card_present();
    if !card_ok {
        log::warn!("no usable card mounted");
    }

    // Capability descriptor for this variant build.
    let caps = board::capabilities(device_id.clone());
    log::info!("variant: {:?} / {:?}", caps.radio, caps.storage);

    // Shared session (also handed to the HTTP server).
    let shared_session: httpd::SharedSession = Arc::new(Mutex::new(None));

    // The app core, shared with BLE callbacks.
    let app: AppShared = Arc::new(NimMutex::new(AppState::new(
        caps,
        device_name,
        device_pubkey,
        auth,
        shared_session.clone(),
        rand32(),
        Box::new(rand16),
        Box::new(now_seconds),
    )));

    // Start BLE GATT + advertising.
    ble::start(app.clone())?;

    // The Wi-Fi radio is created ONCE (the modem can only be taken once) and
    // lives for the whole program; sessions call start_softap/stop on it.
    let mut wifi = wifi::WifiData::new(peripherals.modem, sysloop.clone(), nvs_part.clone())?;

    // ---- Phase 2: event loop ---------------------------------------------
    let mut button_tracker = ui::ButtonTracker::new();
    let mut server: Option<esp_idf_svc::http::server::EspHttpServer<'static>> = None;
    let mut last_status_ms: u32 = 0;

    let now = Arc::new(now_seconds) as httpd::NowFn;

    loop {
        let t_ms = millis();

        // --- button: decode gestures ---
        let is_down = button.is_low(); // active-low
        match button_tracker.sample(is_down, t_ms) {
            ui::ButtonEvent::ShortPress => {
                let mut a = app.lock();
                if a.state() == DeviceState::Asleep {
                    a.apply_event(Event::ButtonShortPress);
                }
                a.open_pairing_window();
            }
            ui::ButtonEvent::LongHold => {
                log::warn!("long hold: factory reset");
                app.lock().do_factory_reset();
            }
            ui::ButtonEvent::None => {}
        }

        // --- periodic status refresh (every ~2s) ---
        if t_ms.wrapping_sub(last_status_ms) >= 2000 {
            last_status_ms = t_ms;
            let battery = power_mon.battery_percent().unwrap_or(0);
            let charging = power::charging_from_stat(chg_stat.is_low());
            let (free, total) = storage::space();
            let snap = StatusSnapshot {
                battery_percent: battery,
                charging,
                free_bytes: free,
                total_bytes: total,
                card_present: storage::card_present(),
            };
            app.lock().update_status(snap);
        }

        // --- drain queued actions (radio + fs work on the main thread) ---
        let pending = app.lock().take_pending();
        for cmd in pending {
            match cmd {
                AppCommand::BringUpData(transport) => {
                    let _ = sd_sel.set_low(); // radio owns the card for wireless
                    if let Err(e) = bring_up_data(
                        transport,
                        &shared_session,
                        &now,
                        &mut wifi,
                        &mut server,
                    ) {
                        log::error!("data bring-up failed: {e}");
                    }
                }
                AppCommand::TearDownData => {
                    server = None; // dropping stops the HTTP server
                    let _ = wifi.stop();
                    let _ = sd_sel.set_low(); // radio reclaims the bus
                    *shared_session.lock().unwrap() = None;
                }
                AppCommand::FactoryReset => app.lock().do_factory_reset(),
                AppCommand::BeginTrickle => {
                    // Trickle decision uses tucklet_core::trickle; the actual
                    // drip is driven by the app over an active data session.
                    log::info!("begin-trickle requested");
                }
                AppCommand::Sleep => {
                    enter_deep_sleep();
                }
            }
        }

        // --- idle -> sleep policy ---
        // (Handled via the COMMAND/IdleTimeout path; a timer could also raise
        // IdleTimeout here after power::IDLE_SLEEP_SECONDS with no connection.)

        FreeRtos::delay_ms(20);
    }
}

// --- data path bring-up (main thread) -------------------------------------

fn bring_up_data(
    transport: DataTransport,
    shared_session: &httpd::SharedSession,
    now: &httpd::NowFn,
    wifi: &mut wifi::WifiData,
    server: &mut Option<esp_idf_svc::http::server::EspHttpServer<'static>>,
) -> anyhow::Result<()> {
    let session = shared_session
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| anyhow::anyhow!("no session to bring up"))?;

    match transport {
        DataTransport::SoftAp => wifi.start_softap(&session)?,
        #[cfg(feature = "wifi_aware")]
        DataTransport::WifiAware => wifi.start_aware(&session)?,
        #[cfg(not(feature = "wifi_aware"))]
        DataTransport::WifiAware => wifi.start_softap(&session)?, // graceful fallback
        DataTransport::WiredUsbHs => {
            // The wired path is owned by the USB-HS bridge, not the radio. The
            // caller has already left SD_SEL such that the bridge can take the
            // bus when VBUS is present; nothing for the radio to do here.
            return Ok(());
        }
    }

    // Start the HTTP data server bound to the session.
    *server = Some(httpd::start(shared_session.clone(), now.clone())?);
    log::info!("data path up ({:?})", transport);
    Ok(())
}

// --- helpers ---------------------------------------------------------------

/// Monotonic milliseconds since boot (esp_timer).
fn millis() -> u32 {
    (unsafe { esp_idf_svc::sys::esp_timer_get_time() } / 1000) as u32
}
/// Monotonic seconds since boot — used for session TTL (relative, so a
/// wall-clock isn't required).
fn now_seconds() -> i64 {
    unsafe { esp_idf_svc::sys::esp_timer_get_time() } / 1_000_000
}

fn rand16() -> [u8; 16] {
    let mut b = [0u8; 16];
    unsafe { esp_idf_svc::sys::esp_fill_random(b.as_mut_ptr() as *mut core::ffi::c_void, b.len()) };
    b
}
fn rand32() -> [u8; 32] {
    let mut b = [0u8; 32];
    unsafe { esp_idf_svc::sys::esp_fill_random(b.as_mut_ptr() as *mut core::ffi::c_void, b.len()) };
    b
}

/// A stable-ish device id from the eFuse MAC.
fn device_id_string() -> String {
    let mut mac = [0u8; 6];
    unsafe {
        esp_idf_svc::sys::esp_read_mac(mac.as_mut_ptr(), esp_idf_svc::sys::esp_mac_type_t_ESP_MAC_WIFI_STA);
    }
    format!(
        "TCK-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

fn enter_deep_sleep() {
    log::info!("entering deep sleep");
    unsafe {
        // Wake on the button GPIO; radios off. (CONFIRM the C5 wake source API.)
        esp_idf_svc::sys::esp_deep_sleep_start();
    }
}
