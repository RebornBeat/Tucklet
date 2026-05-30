// AppModel.swift
// The single observable orchestrator. Owns the BLE control client, the data
// session, the photo-library source, the unified library (on-phone + on-Tucklet),
// transfers, trickle backup, undo, and pairing state.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
import SwiftUI
import UIKit
import CryptoKit
import Security

@MainActor
public final class AppModel: ObservableObject {
    // Connection + device
    @Published public var status: StatusReport?
    @Published public var capabilities: DeviceCapabilities?
    @Published public var connectionState: ConnectionState = .idle
    @Published public var isPaired: Bool = UserDefaults.standard.bool(forKey: "tucklet.isPaired") {
        didSet { UserDefaults.standard.set(isPaired, forKey: "tucklet.isPaired") }
    }

    // Library (the two sources, presented uniformly)
    @Published public var manifest: Manifest?              // items On Tucklet
    @Published public var onPhoneItems: [MediaItem] = []   // items On phone (camera roll)

    // Settings (persisted)
    @AppStorage("defaultTemporaryPolicy") private var defaultTemporaryRaw: String = TemporaryPolicy.oneWeek.rawValue
    @AppStorage("trickleEnabled") public var trickleEnabled: Bool = true
    @AppStorage("trickleOnlyWhileCharging") public var trickleOnlyWhileCharging: Bool = false

    public var defaultTemporaryPolicy: TemporaryPolicy {
        get { TemporaryPolicy(rawValue: defaultTemporaryRaw) ?? .oneWeek }
        set { defaultTemporaryRaw = newValue.rawValue }
    }

    // Undo: ids of the last offload (so "Undo" can bring them back).
    @Published public private(set) var lastOffloadIds: [String] = []

    public enum ConnectionState: Equatable { case idle, connecting, connected, error(String) }

    public let ble = BLEControlClient()
    private lazy var sessions = SessionManager(ble: ble)
    private var data: DataClient?
    private lazy var photos = PhotoKitSource(deviceName: UIDevice.current.name)

    public init() {
        UIDevice.current.isBatteryMonitoringEnabled = true
    }

    // MARK: - Pairing / connection

    /// After the OS-level AccessorySetupKit pairing, perform the cryptographic
    /// enrollment over BLE (PairRequest -> device returns its pubkey), then
    /// connect and load.
    public func completeEnrollmentAndConnect() async {
        do {
            try await ble.connectFirstAvailable()
            let pub = try Crypto.devicePublicKeyHex()
            let resp = try await ble.pair(PairRequest(phonePubkey: pub, phoneName: UIDevice.current.name))
            isPaired = resp.paired
            if resp.paired {
                await connect()
            } else {
                connectionState = .error(resp.reason ?? "pairing not confirmed")
            }
        } catch {
            connectionState = .error(error.localizedDescription)
        }
    }

    /// Silent reconnect to an already-paired charm (no button press), then pull
    /// capabilities + status so the UI is accurate before any transfer.
    public func connect() async {
        connectionState = .connecting
        do {
            try await ble.connectFirstAvailable()
            capabilities = try await ble.readCapabilities()
            status = try await ble.readStatus()
            isPaired = true
            connectionState = .connected
            await refreshPhoneLibrary()
        } catch {
            connectionState = .error(error.localizedDescription)
        }
    }

    /// Open the data session and load the on-Tucklet manifest (metadata only).
    public func loadLibrary() async {
        do {
            let signature = try await Crypto.signSessionChallenge(nonce: ble.lastNonce)
            let client = try await sessions.openDataSession(challengeSignature: signature)
            self.data = client
            self.manifest = try await client.manifest()
        } catch {
            connectionState = .error(error.localizedDescription)
        }
    }

    /// Refresh the on-phone (camera-roll) half of the library.
    public func refreshPhoneLibrary() async {
        guard await photos.requestAccess() else { return }
        onPhoneItems = await photos.cameraRollItems()
    }

    // MARK: - Unified library presentation

    /// All items (both sources) grouped by origin app, for the Library view.
    public func libraryGroups() -> [(app: String, items: [MediaItem])] {
        let all = onPhoneItems + (manifest?.items ?? [])
        let groups = Dictionary(grouping: all, by: { $0.origin.app })
        return groups.keys.sorted().map { (app: $0, items: groups[$0]!.sorted { $0.createdAt > $1.createdAt }) }
    }

    /// Count of photos on the phone not yet backed up to the Tucklet.
    public var pendingBackupCount: Int {
        let onTuckletIds = Set((manifest?.items ?? []).map { $0.id })
        return onPhoneItems.filter { !onTuckletIds.contains($0.id) }.count
    }

    // MARK: - Thumbnails (works for both sources)

    public func thumbnail(for item: MediaItem) async -> UIImage? {
        switch item.state {
        case .onPhone:
            return await photos.thumbnail(for: item.id)
        default:
            if let data = try? await data?.thumbnail(id: item.id) {
                return UIImage(data: data)
            }
            return nil
        }
    }

    // MARK: - Transfers

    public func currentLinkProfile() -> LinkProfile {
        guard let caps = capabilities, let t = sessions.usableTransports(caps).first
        else { return .c5Wireless }
        return LinkProfile.profile(radio: caps.radio, transport: t)
    }

    public func makeTransferEngine() -> TransferEngine? {
        guard let data else { return nil }
        return TransferEngine(data: data, link: currentLinkProfile())
    }

    /// The async per-item worker for an OFFLOAD (free up space): export the
    /// phone original, upload it with its origin, and (if `deleteAfter`) remove
    /// it from the phone. Returns nothing; throws on failure.
    public func offloadItem(_ item: MediaItem) async throws {
        guard let data else { throw TransferError.noSession }
        let url = try await photos.exportOriginal(assetId: item.id)
        try await data.upload(fileURL: url, origin: item.origin, item: item)
        try? FileManager.default.removeItem(at: url)
    }

    /// The async per-item worker for a LOAD (get a copy): download the file to a
    /// temp URL and save it back into Photos (round-trip to its origin album).
    public func loadItem(_ item: MediaItem, policy: TemporaryPolicy) async throws {
        guard let data else { throw TransferError.noSession }
        let dest = FileManager.default.temporaryDirectory.appendingPathComponent(item.id)
        try await data.download(id: item.id, to: dest)
        // For a permanent ("Keep") load, re-import into Photos. Temporary copies
        // are tracked by the app and removed on expiry without touching Photos.
        if policy == .keep {
            try await photos.restoreToPhone(fileURL: dest, origin: item.origin, isVideo: item.isVideo)
        }
    }

    /// After an offload completes, optionally delete the originals from the
    /// phone and record them for Undo.
    public func finishOffload(deletedIds: [String]) async {
        lastOffloadIds = deletedIds
        if !deletedIds.isEmpty {
            try? await photos.deleteFromPhone(assetIds: deletedIds)
            await refreshPhoneLibrary()
        }
        manifest = try? await data?.manifest()
    }

    /// Undo the last offload: pull those items back to the phone.
    public func undoLastOffload() async {
        guard let data, !lastOffloadIds.isEmpty else { return }
        for id in lastOffloadIds {
            if let item = (manifest?.items ?? []).first(where: { $0.id == id }) {
                let dest = FileManager.default.temporaryDirectory.appendingPathComponent(id)
                if (try? await data.download(id: id, to: dest)) != nil {
                    try? await photos.restoreToPhone(fileURL: dest, origin: item.origin, isVideo: item.isVideo)
                }
            }
        }
        lastOffloadIds = []
        await refreshPhoneLibrary()
    }

    /// Restore (put back) a single On-Tucklet item to its origin album.
    public func restore(_ item: MediaItem) async throws {
        guard let data else { throw TransferError.noSession }
        let origin = try await data.restoreOrigin(id: item.id)
        let dest = FileManager.default.temporaryDirectory.appendingPathComponent(item.id)
        try await data.download(id: item.id, to: dest)
        try await photos.restoreToPhone(fileURL: dest, origin: origin, isVideo: item.isVideo)
        await refreshPhoneLibrary()
    }

    public func delete(_ item: MediaItem) async {
        try? await data?.delete(id: item.id)
        manifest = try? await data?.manifest()
    }

    /// "Forget this Tucklet": stop trusting it on THIS phone (disconnect + clear
    /// local pairing). Note: this does not erase this phone from the charm's
    /// allow-list — that requires a factory reset on the device (long button
    /// hold) or a future explicit revoke command. We say so plainly in the UI.
    public func forgetTucklet() async {
        ble.disconnect()
        data = nil
        manifest = nil
        status = nil
        capabilities = nil
        isPaired = false
        connectionState = .idle
    }

    // MARK: - Trickle

    public func trickleConditions() -> Trickle.Conditions {
        let charmBattery = status?.batteryPercent ?? 0
        let phoneCharging = UIDevice.current.batteryState == .charging || UIDevice.current.batteryState == .full
        let phoneIdle = !ProcessInfo.processInfo.isLowPowerModeEnabled
        let charging = trickleOnlyWhileCharging ? phoneCharging : (phoneCharging || phoneIdle)
        return Trickle.Conditions(
            phoneInRange: connectionState == .connected,
            phoneIdle: phoneIdle,
            charging: charging,
            batteryPercent: charmBattery,
            pendingItems: UInt32(pendingBackupCount)
        )
    }

    /// Back up up to `batchSize` not-yet-backed-up photos. Returns the count
    /// actually backed up. Used by both foreground "auto" and the BG task.
    public func trickleBackup(batchSize: Int) async -> Int {
        guard data != nil || (await openSessionIfNeeded()) else { return 0 }
        let onTuckletIds = Set((manifest?.items ?? []).map { $0.id })
        let pending = onPhoneItems.filter { !onTuckletIds.contains($0.id) }.prefix(batchSize)
        var done = 0
        for item in pending {
            do { try await offloadItem(item); done += 1 } catch { break }
        }
        if done > 0 { manifest = try? await data?.manifest() }
        return done
    }

    private func openSessionIfNeeded() async -> Bool {
        if data != nil { return true }
        await loadLibrary()
        return data != nil
    }

    // MARK: - Derived display helpers

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

    public enum TransferError: Error { case noSession }
}

/// Crypto seam. Production: an Ed25519 keypair in the Secure Enclave; the public
/// key is sent at pairing, and each session nonce is signed with the private
/// key. The firmware verifies the signature over the raw nonce with Ed25519.
///
/// Implementation: a Curve25519 (Ed25519) signing key from CryptoKit, persisted
/// in the Keychain (accessible only when the device is unlocked, this device
/// only). Note Secure Enclave can't hold Ed25519 keys (it's P-256 only), so the
/// key is a CryptoKit software key sealed in the Keychain — which is what
/// interoperates with the firmware's Ed25519 verify.
enum Crypto {
    private static let service = "app.tucklet.identity"
    private static let account = "ed25519-secret"

    private static func loadOrCreateKey() throws -> Curve25519.Signing.PrivateKey {
        if let data = try keychainLoad() {
            return try Curve25519.Signing.PrivateKey(rawRepresentation: data)
        }
        let key = Curve25519.Signing.PrivateKey()
        try keychainStore(key.rawRepresentation)
        return key
    }

    /// Hex public key to enroll with the device at pairing (32 bytes -> 64 hex).
    static func devicePublicKeyHex() throws -> String {
        try loadOrCreateKey().publicKey.rawRepresentation.hexString
    }

    /// Sign the RAW nonce bytes (already hex-decoded from the SESSION notify).
    static func signSessionChallenge(nonce: Data?) async throws -> String {
        guard let nonce, !nonce.isEmpty else { throw CryptoError.noNonce }
        let key = try loadOrCreateKey()
        let signature = try key.signature(for: nonce) // Ed25519, 64 bytes
        return signature.hexString
    }

    enum CryptoError: Error { case noNonce, keychain(OSStatus) }

    // MARK: Keychain

    private static func keychainStore(_ data: Data) throws {
        let delete: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        SecItemDelete(delete as CFDictionary)
        var add = delete
        add[kSecValueData as String] = data
        add[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        let status = SecItemAdd(add as CFDictionary, nil)
        guard status == errSecSuccess else { throw CryptoError.keychain(status) }
    }

    private static func keychainLoad() throws -> Data? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var out: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &out)
        if status == errSecItemNotFound { return nil }
        guard status == errSecSuccess else { throw CryptoError.keychain(status) }
        return out as? Data
    }
}

extension Data {
    var hexString: String { map { String(format: "%02x", $0) }.joined() }
}

enum ByteFormat {
    static func short(_ bytes: UInt64) -> String {
        let units = ["B", "KB", "MB", "GB", "TB"]
        var v = Double(bytes); var i = 0
        while v >= 1024 && i < units.count - 1 { v /= 1024; i += 1 }
        return i == 0 ? "\(Int(v)) \(units[i])" : String(format: "%.1f %@", v, units[i])
    }
}
