//! The application core: the shared state every layer talks to, and the
//! handlers for each BLE message. This is where `tucklet-core` (the tested
//! decision logic) meets the on-device glue (BLE/Wi-Fi/storage).
//!
//! Concurrency: wrapped in a NimBLE mutex and shared as [`AppShared`]. BLE
//! callbacks run on the NimBLE host task and lock it briefly; the main loop
//! locks it to drain pending actions and push status. The HTTP server shares
//! only the session (`SharedSession`, a std mutex), not this whole struct.

use esp32_nimble::utilities::mutex::Mutex as NimMutex;
use std::sync::Arc;

use tucklet_core::session::{Session, DEFAULT_TTL_S};
use tucklet_core::state::{DeviceState, Event};
use tucklet_proto::{
    Command, DataTransport, DeviceCapabilities, PairRequest, PairResponse, SessionRequest,
    StatusReport,
};

use crate::auth::AuthStore;
use crate::ble::StatusCharacteristic;
use crate::httpd::SharedSession;

pub type AppShared = Arc<NimMutex<AppState>>;

/// Commands the app raises for the main loop to execute on the main thread
/// (radio + filesystem work that must not run inside a BLE callback).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    BringUpData(DataTransport),
    TearDownData,
    Sleep,
    FactoryReset,
    BeginTrickle,
}

/// The live device status snapshot, refreshed by the main loop from the fuel
/// gauge + card.
#[derive(Debug, Clone)]
pub struct StatusSnapshot {
    pub battery_percent: u8,
    pub charging: bool,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub card_present: bool,
}

impl Default for StatusSnapshot {
    fn default() -> Self {
        Self {
            battery_percent: 0,
            charging: false,
            free_bytes: 0,
            total_bytes: 0,
            card_present: false,
        }
    }
}

pub struct AppState {
    caps: DeviceCapabilities,
    caps_json: String,
    device_name: String,
    device_pubkey_hex: String,

    status: StatusSnapshot,
    state: DeviceState,

    auth: AuthStore,
    shared_session: SharedSession,

    /// True while the physical button has opened a pairing window.
    pairing_window_open: bool,
    /// Fresh per-connection nonce the phone must sign for a silent session.
    current_nonce: [u8; 32],

    /// Queue of actions for the main loop.
    pending: Vec<AppCommand>,

    /// BLE handles for pushing notifications.
    status_notifier: Option<StatusCharacteristic>,
    session_notifier: Option<StatusCharacteristic>,

    /// Injected primitives (hardware-backed in main, so this stays testable).
    rng16: Box<dyn FnMut() -> [u8; 16] + Send>,
    now: Box<dyn Fn() -> i64 + Send>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        caps: DeviceCapabilities,
        device_name: String,
        device_pubkey_hex: String,
        auth: AuthStore,
        shared_session: SharedSession,
        initial_nonce: [u8; 32],
        rng16: Box<dyn FnMut() -> [u8; 16] + Send>,
        now: Box<dyn Fn() -> i64 + Send>,
    ) -> Self {
        let caps_json = serde_json::to_string(&caps).unwrap_or_else(|_| "{}".into());
        Self {
            caps,
            caps_json,
            device_name,
            device_pubkey_hex,
            status: StatusSnapshot::default(),
            state: DeviceState::Advertising,
            auth,
            shared_session,
            pairing_window_open: false,
            current_nonce: initial_nonce,
            pending: Vec::new(),
            status_notifier: None,
            session_notifier: None,
            rng16,
            now,
        }
    }

    // --- wiring from ble.rs ------------------------------------------------

    pub fn set_status_notifier(&mut self, chr: StatusCharacteristic) {
        self.status_notifier = Some(chr);
    }
    pub fn set_session_notifier(&mut self, chr: StatusCharacteristic) {
        self.session_notifier = Some(chr);
    }
    pub fn advertised_name(&self) -> String {
        self.device_name.clone()
    }
    pub fn capabilities_json(&self) -> String {
        self.caps_json.clone()
    }

    // --- BLE connection lifecycle -----------------------------------------

    pub fn on_ble_connected(&mut self) {
        // Fresh nonce per connection; push it so the phone can sign it for a
        // silent session (it signs `current_nonce` with its enrolled key).
        let r = (self.rng16)();
        // expand 16 -> 32 bytes of nonce (two draws would be ideal; this is fine
        // for a challenge nonce as it's single-use and not a key).
        let mut n = [0u8; 32];
        n[..16].copy_from_slice(&r);
        n[16..].copy_from_slice(&(self.rng16)());
        self.current_nonce = n;
        if let Some(sess) = &self.session_notifier {
            let payload = serde_json::json!({ "nonce": hex::encode(self.current_nonce) });
            sess.lock().set_value(payload.to_string().as_bytes()).notify();
        }
        self.apply_event(Event::AuthenticatedPhoneConnected);
    }

    pub fn on_ble_disconnected(&mut self) {
        self.pending.push(AppCommand::TearDownData);
        self.apply_event(Event::PhoneDisconnected);
    }

    // --- BLE message handlers (return JSON bytes to notify back) ----------

    /// AUTH: pair a new phone — only if the button opened a pairing window.
    pub fn handle_auth(&mut self, data: &[u8]) -> Vec<u8> {
        let req: PairRequest = match serde_json::from_slice(data) {
            Ok(r) => r,
            Err(_) => return json(&PairResponse { paired: false, device_pubkey: None, reason: Some("bad request".into()) }),
        };
        if !self.pairing_window_open {
            return json(&PairResponse {
                paired: false,
                device_pubkey: None,
                reason: Some("press the button on Tucklet to confirm".into()),
            });
        }
        match self.auth.enroll(req.phone_pubkey, req.phone_name) {
            Ok(_added) => {
                self.pairing_window_open = false; // single confirmation per press
                json(&PairResponse {
                    paired: true,
                    device_pubkey: Some(self.device_pubkey_hex.clone()),
                    reason: None,
                })
            }
            Err(e) => json(&PairResponse {
                paired: false,
                device_pubkey: None,
                reason: Some(format!("{e}")),
            }),
        }
    }

    /// SESSION: authenticate via challenge-response against any enrolled phone,
    /// then mint single-use transport credentials.
    pub fn handle_session(&mut self, data: &[u8]) -> Vec<u8> {
        let req: SessionRequest = match serde_json::from_slice(data) {
            Ok(r) => r,
            Err(_) => return error_json("bad request"),
        };
        if !self.caps.supports(req.transport) {
            return error_json("unsupported transport");
        }
        // Authenticate: the right phone is whichever enrolled key verifies the
        // signature over the current nonce.
        let authed = self.auth.phones().iter().any(|p| {
            self.auth
                .verify_challenge(&p.pubkey, &self.current_nonce, &req.challenge_signature)
                .unwrap_or(false)
        });
        if !authed {
            return error_json("unauthorized");
        }
        // Mint and publish the session.
        let now = (self.now)();
        let session = Session::mint(req.transport, now, (self.rng16)(), DEFAULT_TTL_S);
        let grant = session.grant();
        *self.shared_session.lock().unwrap() = Some(session);
        self.pending.push(AppCommand::BringUpData(req.transport));
        self.apply_event(Event::SessionStarted);
        json(&grant)
    }

    /// COMMAND: small control ops.
    pub fn handle_command(&mut self, data: &[u8]) {
        let cmd: Command = match serde_json::from_slice(data) {
            Ok(c) => c,
            Err(_) => return,
        };
        match cmd {
            Command::Sleep => {
                self.pending.push(AppCommand::TearDownData);
                self.pending.push(AppCommand::Sleep);
                self.apply_event(Event::IdleTimeout);
            }
            Command::FactoryResetConfirm => self.pending.push(AppCommand::FactoryReset),
            Command::BeginTrickle => self.pending.push(AppCommand::BeginTrickle),
        }
    }

    // --- driven by the main loop ------------------------------------------

    /// Open a pairing window (called when the button is short-pressed).
    pub fn open_pairing_window(&mut self) {
        self.pairing_window_open = true;
        log::info!("pairing window open — write to AUTH to enroll");
    }

    /// Refresh the status snapshot and push a STATUS notification.
    pub fn update_status(&mut self, s: StatusSnapshot) {
        self.status = s;
        if let Some(chr) = &self.status_notifier {
            chr.lock().set_value(&self.status_json_bytes()).notify();
        }
    }

    pub fn status_json(&self) -> String {
        String::from_utf8(self.status_json_bytes()).unwrap_or_default()
    }

    fn status_json_bytes(&self) -> Vec<u8> {
        let report = StatusReport {
            battery_percent: self.status.battery_percent,
            charging: self.status.charging,
            free_bytes: self.status.free_bytes,
            total_bytes: self.status.total_bytes,
            card_present: self.status.card_present,
            firmware_version: self.caps.firmware_version.clone(),
        };
        serde_json::to_vec(&report).unwrap_or_default()
    }

    /// Drain queued actions for the main loop to execute.
    pub fn take_pending(&mut self) -> Vec<AppCommand> {
        std::mem::take(&mut self.pending)
    }

    /// Run a factory reset (clears the allow-list). Called from main after the
    /// long-hold gesture or a FactoryReset command.
    pub fn do_factory_reset(&mut self) {
        if let Err(e) = self.auth.factory_reset() {
            log::error!("factory reset failed: {e}");
        } else {
            log::info!("factory reset: allow-list cleared");
        }
    }

    pub fn state(&self) -> DeviceState {
        self.state
    }

    pub fn apply_event(&mut self, ev: Event) {
        self.state = self.state.next(ev);
    }

    /// Mark the session ended (transfer complete / torn down).
    pub fn end_session(&mut self) {
        *self.shared_session.lock().unwrap() = None;
        self.apply_event(Event::TransferComplete);
    }
}

fn json<T: serde::Serialize>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).unwrap_or_default()
}
fn error_json(msg: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({ "error": msg })).unwrap_or_default()
}
