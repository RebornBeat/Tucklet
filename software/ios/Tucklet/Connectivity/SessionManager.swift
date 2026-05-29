// SessionManager.swift
// Bridges the control plane (BLE) to the data plane (Wi-Fi). Resolves which
// transport to use for this device + platform, brings the network up, and
// hands back a ready-to-use DataClient.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
import NetworkExtension

public enum SessionError: Error { case noUsableTransport, joinFailed(String), wiredNotOnIOS }

@MainActor
public final class SessionManager {
    private let ble: BLEControlClient
    public init(ble: BLEControlClient) { self.ble = ble }

    /// Mirror of `tucklet-core::variant::usable_transports` for the iOS client.
    public func usableTransports(_ caps: DeviceCapabilities) -> [DataTransport] {
        var out: [DataTransport] = []
        if caps.supports(.wifiAware) { out.append(.wifiAware) }   // iOS 26+ supports Aware
        if caps.supports(.softAp) { out.append(.softAp) }
        // Wired (USB-C bridge) appears as a mass-storage device handled by the
        // Files app / File Provider, not through this Wi-Fi session manager.
        return out
    }

    /// Open a data session: pick transport, get one-time creds over BLE, join.
    public func openDataSession(challengeSignature: String) async throws -> DataClient {
        guard let caps = ble.capabilities else { throw SessionError.noUsableTransport }
        guard let transport = usableTransports(caps).first else { throw SessionError.noUsableTransport }

        let grant = try await ble.openSession(
            SessionRequest(challengeSignature: challengeSignature, transport: transport))

        switch transport {
        case .softAp:
            try await joinSoftAP(ssid: grant.ssidOrService, psk: grant.psk)
        case .wifiAware:
            try await joinWifiAware(service: grant.ssidOrService, passphrase: grant.psk)
        case .wiredUsbHs:
            throw SessionError.wiredNotOnIOS
        }

        let base = URL(string: "http://\(grant.ip)")!
        return DataClient(baseURL: base, token: grant.token)
    }

    /// Join the device's one-time SoftAP. This shows the OS "join Wi-Fi?" prompt
    /// the first time; subsequent joins of the same SSID are smoother.
    private func joinSoftAP(ssid: String, psk: String) async throws {
        let cfg = NEHotspotConfiguration(ssid: ssid, passphrase: psk, isWEP: false)
        cfg.joinOnce = true   // single-use network; don't persist
        do {
            try await NEHotspotConfigurationManager.shared.apply(cfg)
        } catch {
            // "already associated" is success, not failure.
            let ns = error as NSError
            if ns.domain == NEHotspotConfigurationErrorDomain,
               ns.code == NEHotspotConfigurationError.alreadyAssociated.rawValue { return }
            throw SessionError.joinFailed(error.localizedDescription)
        }
    }

    /// Wi-Fi Aware peer-to-peer (iOS 26). Seamless: no "join network" prompt.
    /// CONFIRM the exact WiFiAware framework API (publisher/subscriber, paired
    /// device handle, data path establishment) against the iOS 26 SDK — the
    /// shape below reflects the WWDC25 model but the symbol names must be
    /// verified before this compiles.
    private func joinWifiAware(service: String, passphrase: String) async throws {
        // import WiFiAware
        // let device = try await WAPairedDevice.resolve(...)            // CONFIRM
        // let session = try await WADataSession.connect(to: device,     // CONFIRM
        //                                               service: service,
        //                                               passphrase: passphrase)
        // The DataClient then targets the link-local endpoint the session vends.
        throw SessionError.joinFailed("WiFiAware path pending SDK confirmation")
    }
}
