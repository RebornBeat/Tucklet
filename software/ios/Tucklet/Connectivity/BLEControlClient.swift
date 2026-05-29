// BLEControlClient.swift
// Control plane over BLE (CoreBluetooth). Small JSON messages only — discovery,
// authentication, status, and opening a transfer session. Bulk data never
// touches BLE (see PROTOCOL.md).
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
@preconcurrency import CoreBluetooth

/// 128-bit UUIDs — generate real ones for production (these are placeholders
/// that must match the firmware's GATT table).
enum TuckletGATT {
    static let service     = CBUUID(string: "F0CC0001-0000-1000-8000-00805F9B34FB")
    static let statusChar  = CBUUID(string: "F0CC0002-0000-1000-8000-00805F9B34FB")
    static let authChar    = CBUUID(string: "F0CC0003-0000-1000-8000-00805F9B34FB")
    static let sessionChar = CBUUID(string: "F0CC0004-0000-1000-8000-00805F9B34FB")
    static let commandChar = CBUUID(string: "F0CC0005-0000-1000-8000-00805F9B34FB")
    static let capsChar    = CBUUID(string: "F0CC0006-0000-1000-8000-00805F9B34FB")
}

public enum BLEError: Error { case poweredOff, notConnected, timeout, charMissing, decode }

@MainActor
public final class BLEControlClient: NSObject, ObservableObject {
    @Published public private(set) var isConnected = false
    @Published public private(set) var lastStatus: StatusReport?
    @Published public private(set) var capabilities: DeviceCapabilities?

    private var central: CBCentralManager!
    private var peripheral: CBPeripheral?
    private var chars: [CBUUID: CBCharacteristic] = [:]

    // Async plumbing: one-shot continuations keyed by characteristic.
    private var poweredOn: CheckedContinuation<Void, Error>?
    private var connectCont: CheckedContinuation<Void, Error>?
    private var readConts: [CBUUID: CheckedContinuation<Data, Error>] = [:]

    public override init() {
        super.init()
        central = CBCentralManager(delegate: self, queue: .main)
    }

    // MARK: Public API

    /// Scan for the first Tucklet advertising and connect to it. For known
    /// devices this is silent (no button press); pairing of a NEW device is
    /// handled by AccessorySetupKit at the OS level before this runs.
    public func connectFirstAvailable(timeout: TimeInterval = 10) async throws {
        try await waitPoweredOn()
        central.scanForPeripherals(withServices: [TuckletGATT.service])
        try await withTimeout(timeout) {
            try await withCheckedThrowingContinuation { (c: CheckedContinuation<Void, Error>) in
                self.connectCont = c
            }
        }
    }

    public func disconnect() {
        if let p = peripheral { central.cancelPeripheralConnection(p) }
    }

    /// Read the device capability descriptor (the variant matrix this unit is).
    public func readCapabilities() async throws -> DeviceCapabilities {
        let data = try await read(TuckletGATT.capsChar)
        let caps = try JSONDecoder().decode(DeviceCapabilities.self, from: data)
        self.capabilities = caps
        return caps
    }

    public func readStatus() async throws -> StatusReport {
        let data = try await read(TuckletGATT.statusChar)
        let s = try JSONDecoder().decode(StatusReport.self, from: data)
        self.lastStatus = s
        return s
    }

    /// First-time pairing write (the rest of pairing UX is AccessorySetupKit).
    public func pair(_ req: PairRequest) async throws -> PairResponse {
        try write(try JSONEncoder().encode(req), to: TuckletGATT.authChar)
        let data = try await read(TuckletGATT.authChar)   // device notifies the result
        return try JSONDecoder().decode(PairResponse.self, from: data)
    }

    /// Open a transfer session; device returns single-use transport credentials.
    public func openSession(_ req: SessionRequest) async throws -> SessionGrant {
        try write(try JSONEncoder().encode(req), to: TuckletGATT.sessionChar)
        let data = try await read(TuckletGATT.sessionChar)
        return try JSONDecoder().decode(SessionGrant.self, from: data)
    }

    public func send(_ command: Data) throws {
        try write(command, to: TuckletGATT.commandChar)
    }

    // MARK: Internals

    private func waitPoweredOn() async throws {
        if central.state == .poweredOn { return }
        if central.state == .poweredOff { throw BLEError.poweredOff }
        try await withCheckedThrowingContinuation { (c: CheckedContinuation<Void, Error>) in
            self.poweredOn = c
        }
    }

    private func read(_ uuid: CBUUID) async throws -> Data {
        guard let p = peripheral, let ch = chars[uuid] else { throw BLEError.notConnected }
        return try await withTimeout(8) {
            try await withCheckedThrowingContinuation { (c: CheckedContinuation<Data, Error>) in
                self.readConts[uuid] = c
                p.readValue(for: ch)
            }
        }
    }

    private func write(_ data: Data, to uuid: CBUUID) throws {
        guard let p = peripheral, let ch = chars[uuid] else { throw BLEError.notConnected }
        p.writeValue(data, for: ch, type: .withResponse)
    }

    private func withTimeout<T: Sendable>(_ seconds: TimeInterval,
                                          _ op: @escaping @Sendable () async throws -> T) async throws -> T {
        try await withThrowingTaskGroup(of: T.self) { group in
            group.addTask { try await op() }
            group.addTask {
                try await Task.sleep(nanoseconds: UInt64(seconds * 1_000_000_000))
                throw BLEError.timeout
            }
            let result = try await group.next()!
            group.cancelAll()
            return result
        }
    }
}

// CoreBluetooth delegate. Marked nonisolated and hops to the main actor; the
// CBCentralManager queue is .main so this is consistent.
extension BLEControlClient: CBCentralManagerDelegate, CBPeripheralDelegate {
    public nonisolated func centralManagerDidUpdateState(_ central: CBCentralManager) {
        Task { @MainActor in
            switch central.state {
            case .poweredOn: self.poweredOn?.resume(); self.poweredOn = nil
            case .poweredOff: self.poweredOn?.resume(throwing: BLEError.poweredOff); self.poweredOn = nil
            default: break
            }
        }
    }

    public nonisolated func centralManager(_ central: CBCentralManager,
                                           didDiscover peripheral: CBPeripheral,
                                           advertisementData: [String: Any], rssi RSSI: NSNumber) {
        Task { @MainActor in
            central.stopScan()
            self.peripheral = peripheral
            peripheral.delegate = self
            central.connect(peripheral)
        }
    }

    public nonisolated func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        Task { @MainActor in peripheral.discoverServices([TuckletGATT.service]) }
    }

    public nonisolated func centralManager(_ central: CBCentralManager,
                                           didDisconnectPeripheral peripheral: CBPeripheral, error: Error?) {
        Task { @MainActor in self.isConnected = false; self.chars.removeAll() }
    }

    public nonisolated func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        Task { @MainActor in
            guard let svc = peripheral.services?.first(where: { $0.uuid == TuckletGATT.service }) else { return }
            peripheral.discoverCharacteristics(nil, for: svc)
        }
    }

    public nonisolated func peripheral(_ peripheral: CBPeripheral,
                                       didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        Task { @MainActor in
            for ch in service.characteristics ?? [] {
                self.chars[ch.uuid] = ch
                if ch.properties.contains(.notify) { peripheral.setNotifyValue(true, for: ch) }
            }
            self.isConnected = true
            self.connectCont?.resume(); self.connectCont = nil
        }
    }

    public nonisolated func peripheral(_ peripheral: CBPeripheral,
                                       didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        Task { @MainActor in
            let uuid = characteristic.uuid
            if let cont = self.readConts.removeValue(forKey: uuid) {
                if let e = error { cont.resume(throwing: e) }
                else { cont.resume(returning: characteristic.value ?? Data()) }
            }
            // Live STATUS notifications keep the published value fresh.
            if uuid == TuckletGATT.statusChar, let v = characteristic.value,
               let s = try? JSONDecoder().decode(StatusReport.self, from: v) {
                self.lastStatus = s
            }
        }
    }
}
