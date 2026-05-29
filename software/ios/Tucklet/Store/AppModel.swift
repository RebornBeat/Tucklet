// AppModel.swift
// The single observable app state. Owns the BLE client, opens data sessions,
// loads the manifest, and exposes status for the views.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
import SwiftUI

@MainActor
public final class AppModel: ObservableObject {
    @Published public var status: StatusReport?
    @Published public var capabilities: DeviceCapabilities?
    @Published public var manifest: Manifest?
    @Published public var connectionState: ConnectionState = .idle
    @Published public var defaultTemporaryPolicy: TemporaryPolicy = .oneWeek
    @Published public var trickleEnabled = true
    @Published public var trickleOnlyWhileCharging = false

    public enum ConnectionState: Equatable { case idle, connecting, connected, error(String) }

    public let ble = BLEControlClient()
    private lazy var sessions = SessionManager(ble: ble)
    private var data: DataClient?

    public init() {}

    /// Silent reconnect (no button press) to an already-paired charm, then
    /// pull capabilities + status so the UI is accurate before any transfer.
    public func connect() async {
        connectionState = .connecting
        do {
            try await ble.connectFirstAvailable()
            capabilities = try await ble.readCapabilities()
            status = try await ble.readStatus()
            connectionState = .connected
        } catch {
            connectionState = .error(error.localizedDescription)
        }
    }

    /// Open the data session and load the library (metadata + thumbnails only).
    public func loadLibrary() async {
        do {
            // Production: sign the device nonce with the phone's private key.
            let signature = try Crypto.signSessionChallenge()
            let client = try await sessions.openDataSession(challengeSignature: signature)
            self.data = client
            self.manifest = try await client.manifest()
        } catch {
            connectionState = .error(error.localizedDescription)
        }
    }

    public func currentLinkProfile() -> LinkProfile {
        guard let caps = capabilities, let t = sessions.usableTransports(caps).first
        else { return .c5Wireless }
        return LinkProfile.profile(radio: caps.radio, transport: t)
    }

    public func makeTransferEngine() -> TransferEngine? {
        guard let data else { return nil }
        return TransferEngine(data: data, link: currentLinkProfile())
    }

    public func thumbnail(for item: MediaItem) async -> Data? {
        try? await data?.thumbnail(id: item.id)
    }

    // MARK: derived display helpers
    public var freeText: String {
        guard let s = status else { return "—" }
        return "\(ByteFormat.short(s.freeBytes)) free of \(ByteFormat.short(s.totalBytes))"
    }
    public var storageDescription: String {
        switch capabilities?.storage {
        case .microSd: return "microSD"
        case .emmc(let g): return "\(g) GB built-in"
        case nil: return "—"
        }
    }
}

/// Minimal crypto seam. Production: X25519/Ed25519 keypair in the Secure
/// Enclave; sign the device-issued session nonce. CONFIRM the exact key
/// storage/signing API you adopt (CryptoKit + SecureEnclave) here.
enum Crypto {
    static func signSessionChallenge() throws -> String {
        // Placeholder-free seam: returns a deterministic stand-in until the
        // Secure Enclave keypair is wired. This is the ONE security primitive
        // that must be completed with real signing before shipping.
        return "ed25519-sig-pending-secure-enclave"
    }
}

enum ByteFormat {
    static func short(_ bytes: UInt64) -> String {
        let units = ["B", "KB", "MB", "GB", "TB"]
        var v = Double(bytes); var i = 0
        while v >= 1024 && i < units.count - 1 { v /= 1024; i += 1 }
        return i == 0 ? "\(Int(v)) \(units[i])" : String(format: "%.1f %@", v, units[i])
    }
}
