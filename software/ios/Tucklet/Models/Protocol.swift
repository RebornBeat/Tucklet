// Protocol.swift
// Swift mirror of `tucklet-proto` (the Rust source of truth). These Codable
// types decode/encode the exact same JSON the firmware emits, so the app and
// device always agree. Keep this in lockstep with firmware/crates/tucklet-proto.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation

public let protocolVersion: UInt16 = 1
public typealias EpochSeconds = Int64

// MARK: - Variant matrix

public enum RadioKind: String, Codable, Sendable { case singleC5 = "single_c5", dualC5 = "dual_c5" }

public enum StorageKind: Codable, Sendable, Equatable {
    case microSd
    case emmc(capacityGiB: UInt32)

    private enum CodingKeys: String, CodingKey { case microSd = "micro_sd", emmc }
    private struct EmmcPayload: Codable { let capacity_gib: UInt32 }

    public init(from decoder: Decoder) throws {
        // serde encodes externally-tagged enums: {"micro_sd": null} or {"emmc":{"capacity_gib":64}}
        let c = try decoder.container(keyedBy: CodingKeys.self)
        if c.contains(.emmc) {
            let p = try c.decode(EmmcPayload.self, forKey: .emmc)
            self = .emmc(capacityGiB: p.capacity_gib)
        } else {
            self = .microSd
        }
    }
    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .microSd: try c.encodeNil(forKey: .microSd)
        case .emmc(let cap): try c.encode(EmmcPayload(capacity_gib: cap), forKey: .emmc)
        }
    }
}

public enum DataTransport: String, Codable, Sendable, CaseIterable {
    case softAp = "soft_ap"
    case wifiAware = "wifi_aware"
    case wiredUsbHs = "wired_usb_hs"
}

public struct DeviceCapabilities: Codable, Sendable, Equatable {
    public let protocolVersion: UInt16
    public let firmwareVersion: String
    public let radio: RadioKind
    public let storage: StorageKind
    public let transports: [DataTransport]
    public let deviceId: String

    enum CodingKeys: String, CodingKey {
        case protocolVersion = "protocol_version"
        case firmwareVersion = "firmware_version"
        case radio, storage, transports
        case deviceId = "device_id"
    }

    public func supports(_ t: DataTransport) -> Bool { transports.contains(t) }

    /// Best wireless transport, preferring seamless Wi-Fi Aware over SoftAP.
    public var bestWireless: DataTransport? {
        if supports(.wifiAware) { return .wifiAware }
        if supports(.softAp) { return .softAp }
        return nil
    }
}

// MARK: - Item state (plain-language UX vocabulary; never "cache")

public enum ItemState: Codable, Sendable, Equatable {
    case onPhone
    case onTucklet
    case temporary(expiresAt: EpochSeconds?)

    private enum CodingKeys: String, CodingKey { case state, expires_at }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        switch try c.decode(String.self, forKey: .state) {
        case "on_phone": self = .onPhone
        case "on_tucklet": self = .onTucklet
        case "temporary": self = .temporary(expiresAt: try c.decodeIfPresent(EpochSeconds.self, forKey: .expires_at))
        case let other: throw DecodingError.dataCorruptedError(forKey: .state, in: c, debugDescription: "unknown state \(other)")
        }
    }
    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .onPhone: try c.encode("on_phone", forKey: .state)
        case .onTucklet: try c.encode("on_tucklet", forKey: .state)
        case .temporary(let e):
            try c.encode("temporary", forKey: .state)
            try c.encodeIfPresent(e, forKey: .expires_at)
        }
    }

    public var label: String {
        switch self {
        case .onPhone: return "On phone"
        case .onTucklet: return "On Tucklet"
        case .temporary: return "Temporary"
        }
    }
}

public enum TemporaryPolicy: String, Codable, Sendable, CaseIterable {
    case oneHour = "one_hour", oneDay = "one_day", oneWeek = "one_week", keep

    public var lifetimeSeconds: Int64? {
        switch self {
        case .oneHour: return 3_600
        case .oneDay: return 86_400
        case .oneWeek: return 604_800
        case .keep: return nil
        }
    }
    public var label: String {
        switch self {
        case .oneHour: return "1 hour"; case .oneDay: return "1 day"
        case .oneWeek: return "1 week"; case .keep: return "Keep"
        }
    }
    public func resolve(now: EpochSeconds) -> ItemState {
        .temporary(expiresAt: lifetimeSeconds.map { now + $0 })
    }
}

// MARK: - Origin + media

public enum Platform: String, Codable, Sendable { case android, ios, desktop }

public struct OriginMetadata: Codable, Sendable, Equatable {
    public let platform: Platform
    public let app: String
    public let collection: String
    public let album: String?
    public let deviceName: String
    enum CodingKeys: String, CodingKey { case platform, app, collection, album, deviceName = "device_name" }
}

public struct MediaItem: Codable, Sendable, Equatable, Identifiable {
    public let id: String
    public let name: String
    public let sizeBytes: UInt64
    public let mime: String
    public let createdAt: EpochSeconds
    public let origin: OriginMetadata
    public let state: ItemState
    public let checksum: String?

    // `state` is flattened to the top level to match the firmware's
    // #[serde(flatten)] (JSON carries "state" and optional "expires_at"
    // alongside the other fields, not as a nested object).
    enum CodingKeys: String, CodingKey {
        case id, name, sizeBytes = "size_bytes", mime, createdAt = "created_at"
        case origin, checksum
        case state, expires_at
    }

    public init(id: String, name: String, sizeBytes: UInt64, mime: String,
                createdAt: EpochSeconds, origin: OriginMetadata, state: ItemState,
                checksum: String?) {
        self.id = id; self.name = name; self.sizeBytes = sizeBytes; self.mime = mime
        self.createdAt = createdAt; self.origin = origin; self.state = state; self.checksum = checksum
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        id = try c.decode(String.self, forKey: .id)
        name = try c.decode(String.self, forKey: .name)
        sizeBytes = try c.decode(UInt64.self, forKey: .sizeBytes)
        mime = try c.decode(String.self, forKey: .mime)
        createdAt = try c.decode(EpochSeconds.self, forKey: .createdAt)
        origin = try c.decode(OriginMetadata.self, forKey: .origin)
        checksum = try c.decodeIfPresent(String.self, forKey: .checksum)
        // Flattened state, read from the same container.
        switch try c.decode(String.self, forKey: .state) {
        case "on_phone": state = .onPhone
        case "on_tucklet": state = .onTucklet
        case "temporary": state = .temporary(expiresAt: try c.decodeIfPresent(EpochSeconds.self, forKey: .expires_at))
        case let other: throw DecodingError.dataCorruptedError(forKey: .state, in: c, debugDescription: "unknown state \(other)")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        try c.encode(id, forKey: .id)
        try c.encode(name, forKey: .name)
        try c.encode(sizeBytes, forKey: .sizeBytes)
        try c.encode(mime, forKey: .mime)
        try c.encode(createdAt, forKey: .createdAt)
        try c.encode(origin, forKey: .origin)
        try c.encodeIfPresent(checksum, forKey: .checksum)
        switch state {
        case .onPhone: try c.encode("on_phone", forKey: .state)
        case .onTucklet: try c.encode("on_tucklet", forKey: .state)
        case .temporary(let e):
            try c.encode("temporary", forKey: .state)
            try c.encodeIfPresent(e, forKey: .expires_at)
        }
    }

    public var isImage: Bool { mime.hasPrefix("image/") }
    public var isVideo: Bool { mime.hasPrefix("video/") }
}

public struct Manifest: Codable, Sendable, Equatable {
    public let items: [MediaItem]
    public let freeBytes: UInt64
    public let totalBytes: UInt64
    enum CodingKeys: String, CodingKey { case items, freeBytes = "free_bytes", totalBytes = "total_bytes" }
}

// MARK: - Control plane

public struct StatusReport: Codable, Sendable, Equatable {
    public let batteryPercent: UInt8
    public let charging: Bool
    public let freeBytes: UInt64
    public let totalBytes: UInt64
    public let cardPresent: Bool
    public let firmwareVersion: String
    enum CodingKeys: String, CodingKey {
        case batteryPercent = "battery_percent", charging
        case freeBytes = "free_bytes", totalBytes = "total_bytes"
        case cardPresent = "card_present", firmwareVersion = "firmware_version"
    }
}

public struct PairRequest: Codable, Sendable {
    public let phonePubkey: String
    public let phoneName: String
    enum CodingKeys: String, CodingKey { case phonePubkey = "phone_pubkey", phoneName = "phone_name" }
}

public struct PairResponse: Codable, Sendable {
    public let paired: Bool
    public let devicePubkey: String?
    public let reason: String?
    enum CodingKeys: String, CodingKey { case paired, devicePubkey = "device_pubkey", reason }
}

public struct SessionRequest: Codable, Sendable {
    public let challengeSignature: String
    public let transport: DataTransport
    enum CodingKeys: String, CodingKey { case challengeSignature = "challenge_signature", transport }
}

public struct SessionGrant: Codable, Sendable {
    public let ssidOrService: String
    public let psk: String
    public let ip: String
    public let token: String
    public let ttlSeconds: UInt32
    enum CodingKeys: String, CodingKey {
        case ssidOrService = "ssid_or_service", psk, ip, token, ttlSeconds = "ttl_seconds"
    }
}

// MARK: - Transfers

public enum TransferKind: String, Codable, Sendable { case offload, load }
public enum TransferMode: String, Codable, Sendable { case batch, trickle }

public struct TransferItem: Codable, Sendable, Equatable {
    public let id: String
    public let sizeBytes: UInt64
    public let mime: String
    enum CodingKeys: String, CodingKey { case id, sizeBytes = "size_bytes", mime }
}

public struct TransferRequest: Codable, Sendable {
    public let kind: TransferKind
    public let mode: TransferMode
    public let items: [TransferItem]
    public let temporaryPolicy: TemporaryPolicy?
    enum CodingKeys: String, CodingKey { case kind, mode, items, temporaryPolicy = "temporary_policy" }
}

public struct TransferProgress: Codable, Sendable, Equatable {
    public let itemsTotal: UInt32
    public let itemsDone: UInt32
    public let bytesTotal: UInt64
    public let bytesDone: UInt64
    public let etaSeconds: UInt32
    public let throughputBps: UInt64
    enum CodingKeys: String, CodingKey {
        case itemsTotal = "items_total", itemsDone = "items_done"
        case bytesTotal = "bytes_total", bytesDone = "bytes_done"
        case etaSeconds = "eta_seconds", throughputBps = "throughput_bps"
    }
}
