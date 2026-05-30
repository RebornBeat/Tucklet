//! BLE control plane (NimBLE GATT). Carries only small JSON messages —
//! discovery, capability read, the pairing handshake, status notifications, and
//! the transfer-session request. Bulk data never touches BLE.
//!
//! One service with five characteristics:
//!   CAPS     (read)          DeviceCapabilities JSON
//!   STATUS   (read/notify)   StatusReport JSON
//!   AUTH     (write/notify)  PairRequest -> PairResponse (button-gated)
//!   SESSION  (write/notify)  SessionRequest -> SessionGrant (after challenge)
//!   COMMAND  (write)         Command JSON (sleep / factory-reset / begin-trickle)
//!
//! CONFIRM: the exact esp32-nimble closure signatures vary by crate version
//! (0.8 here). The structure below matches 0.8's `on_read` / `on_write` shapes;
//! re-check against your pinned version when you build.

use esp32_nimble::utilities::mutex::Mutex as NimMutex;
use esp32_nimble::{
    BLEAdvertisementData, BLEDevice, NimbleProperties,
};
use std::sync::Arc;

use crate::app::{AppCommand, AppShared};

// 128-bit UUIDs — replace with freshly-generated UUIDs before release. They
// must match the app's `TuckletGATT` constants (software/ios/.../BLEControlClient.swift).
pub const SVC_UUID: &str = "f0cc0001-0000-1000-8000-00805f9b34fb";
pub const CHR_STATUS: &str = "f0cc0002-0000-1000-8000-00805f9b34fb";
pub const CHR_AUTH: &str = "f0cc0003-0000-1000-8000-00805f9b34fb";
pub const CHR_SESSION: &str = "f0cc0004-0000-1000-8000-00805f9b34fb";
pub const CHR_COMMAND: &str = "f0cc0005-0000-1000-8000-00805f9b34fb";
pub const CHR_CAPS: &str = "f0cc0006-0000-1000-8000-00805f9b34fb";

use esp32_nimble::uuid128;

/// Start the GATT server and begin advertising. Wiring all five characteristics
/// to the shared application state.
pub fn start(app: AppShared) -> anyhow::Result<()> {
    let device = BLEDevice::take();
    let server = device.get_server();

    // Log connect/disconnect so the state machine can react (handled in main via
    // app shared flags).
    {
        let app = app.clone();
        server.on_connect(move |_srv, desc| {
            log::info!("BLE connected: {:?}", desc);
            app.lock().on_ble_connected();
        });
    }
    {
        let app = app.clone();
        server.on_disconnect(move |_desc, _reason| {
            log::info!("BLE disconnected");
            app.lock().on_ble_disconnected();
        });
    }

    let service = server.create_service(uuid128!(SVC_UUID));

    // --- CAPS (read) -------------------------------------------------------
    {
        let caps_chr = service
            .lock()
            .create_characteristic(uuid128!(CHR_CAPS), NimbleProperties::READ);
        let app = app.clone();
        caps_chr.lock().on_read(move |attr, _| {
            let json = app.lock().capabilities_json();
            attr.set_value(json.as_bytes());
        });
    }

    // --- STATUS (read + notify) -------------------------------------------
    let status_chr = service.lock().create_characteristic(
        uuid128!(CHR_STATUS),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    {
        let app = app.clone();
        status_chr.lock().on_read(move |attr, _| {
            let json = app.lock().status_json();
            attr.set_value(json.as_bytes());
        });
    }
    // Hand the status characteristic to the app so it can push notifications
    // (battery/free-space changes) without a read.
    app.lock().set_status_notifier(status_chr.clone());

    // --- AUTH (write + notify) --------------------------------------------
    {
        let auth_chr = service.lock().create_characteristic(
            uuid128!(CHR_AUTH),
            NimbleProperties::WRITE | NimbleProperties::NOTIFY,
        );
        let app = app.clone();
        let auth_for_notify = auth_chr.clone();
        auth_chr.lock().on_write(move |args| {
            let data = args.recv_data().to_vec();
            let resp = app.lock().handle_auth(&data);
            auth_for_notify.lock().set_value(&resp).notify();
        });
    }

    // --- SESSION (write + notify) -----------------------------------------
    {
        let sess_chr = service.lock().create_characteristic(
            uuid128!(CHR_SESSION),
            NimbleProperties::WRITE | NimbleProperties::NOTIFY,
        );
        let app = app.clone();
        let sess_for_notify = sess_chr.clone();
        sess_chr.lock().on_write(move |args| {
            let data = args.recv_data().to_vec();
            let resp = app.lock().handle_session(&data);
            sess_for_notify.lock().set_value(&resp).notify();
        });
    }

    // --- COMMAND (write) ---------------------------------------------------
    {
        let cmd_chr = service
            .lock()
            .create_characteristic(uuid128!(CHR_COMMAND), NimbleProperties::WRITE);
        let app = app.clone();
        cmd_chr.lock().on_write(move |args| {
            let data = args.recv_data().to_vec();
            app.lock().handle_command(&data);
        });
    }

    // --- Advertising -------------------------------------------------------
    let adv = device.get_advertising();
    let device_name = app.lock().advertised_name();
    adv.lock()
        .set_data(
            BLEAdvertisementData::new()
                .name(&device_name)
                .add_service_uuid(uuid128!(SVC_UUID)),
        )?;
    // Low duty cycle: cheap to keep discoverable to a paired phone (see
    // FINAL_REVIEW §3 — tens-to-hundreds of microamps average).
    adv.lock().start()?;
    log::info!("BLE advertising as {device_name}");

    Ok(())
}

/// Re-export so main can hold the notifier type without importing nimble there.
pub type StatusCharacteristic = Arc<NimMutex<esp32_nimble::BLECharacteristic>>;

/// Bridge type used by the app to issue a one-shot command to the BLE layer.
/// (Kept minimal; the app pushes commands into its own queue consumed in main.)
pub type CommandSink = std::sync::mpsc::Sender<AppCommand>;
