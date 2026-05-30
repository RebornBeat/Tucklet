// discovery.rs
// Finding a Tucklet from a computer. The control plane is BLE (same GATT service
// as the firmware); the data plane is Wi-Fi. On the wired path the device simply
// mounts as a USB Mass Storage drive — no app needed — so this module is only
// the wireless discovery seam.
//
// The trait keeps the rest of the app testable without a Bluetooth stack; the
// real btleplug-backed implementation is behind the `ble` feature so the default
// build/test stays free of system Bluetooth libraries.
//
// License: PolyForm Noncommercial 1.0.0

/// A discovered Tucklet we can connect to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceHandle {
    pub id: String,   // platform-specific address/UUID
    pub name: String, // advertised name
}

/// Anything that can find paired/nearby Tucklets.
pub trait Discovery {
    fn scan(&self, timeout_ms: u32) -> Result<Vec<DeviceHandle>, String>;
}

/// The BLE service UUID advertised by the firmware (matches ble.rs SVC_UUID).
pub const SERVICE_UUID: &str = "F0CC0001-0000-1000-8000-00805F9B34FB";

/// A stub used by tests and by the wired-only path.
pub struct NullDiscovery;
impl Discovery for NullDiscovery {
    fn scan(&self, _timeout_ms: u32) -> Result<Vec<DeviceHandle>, String> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "ble")]
pub mod bluetooth {
    //! Real cross-platform BLE discovery via btleplug.
    //! CONFIRM on target OS: adapter permissions (macOS Bluetooth entitlement,
    //! Linux BlueZ running, Windows WinRT), and that the firmware advertises the
    //! 128-bit service UUID in its advertisement (not just the GATT table).
    use super::*;
    use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
    use btleplug::platform::Manager;
    use std::time::Duration;
    use uuid::Uuid;

    pub struct BleDiscovery;

    impl Discovery for BleDiscovery {
        fn scan(&self, timeout_ms: u32) -> Result<Vec<DeviceHandle>, String> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            rt.block_on(async move {
                let manager = Manager::new().await.map_err(|e| e.to_string())?;
                let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
                let central = adapters.into_iter().next().ok_or("no BT adapter")?;
                let svc = Uuid::parse_str(SERVICE_UUID).unwrap();
                central
                    .start_scan(ScanFilter { services: vec![svc] })
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::time::sleep(Duration::from_millis(timeout_ms as u64)).await;
                let mut out = Vec::new();
                for p in central.peripherals().await.map_err(|e| e.to_string())? {
                    if let Ok(Some(props)) = p.properties().await {
                        out.push(DeviceHandle {
                            id: p.id().to_string(),
                            name: props.local_name.unwrap_or_else(|| "Tucklet".into()),
                        });
                    }
                }
                Ok(out)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_discovery_finds_nothing() {
        assert!(NullDiscovery.scan(500).unwrap().is_empty());
    }

    #[test]
    fn service_uuid_matches_firmware() {
        assert!(SERVICE_UUID.starts_with("F0CC0001"));
    }
}
