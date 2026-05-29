// TransferEngine.swift
// Runs transfers and reports honest, live progress. Pre-transfer estimate comes
// from TransferEstimator; the live ETA is recomputed from measured throughput
// so the number stays truthful if the link slows.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation

@MainActor
public final class TransferEngine: ObservableObject {
    @Published public private(set) var progress: TransferProgress?
    @Published public private(set) var isRunning = false
    @Published public private(set) var lastError: String?

    private let data: DataClient
    private let link: LinkProfile
    public init(data: DataClient, link: LinkProfile) { self.data = data; self.link = link }

    /// Estimate before starting (what the confirmation sheet shows).
    public func estimate(items: [TransferItem]) -> TransferEstimate {
        TransferEstimator.estimate(sizes: items.map(\.sizeBytes), link: link)
    }

    /// Run a batch transfer, updating `progress` (with a live ETA) as it goes.
    /// `loadDestination` provides a local file URL for each Load item; for
    /// Offload, `offloadSource` provides the local file to upload.
    public func runBatch(_ request: TransferRequest,
                         offloadSource: ((TransferItem) -> URL)? = nil,
                         loadDestination: ((TransferItem) -> URL)? = nil,
                         origin: OriginMetadata? = nil) async {
        isRunning = true; lastError = nil
        defer { isRunning = false }

        let bytesTotal = request.items.reduce(UInt64(0)) { $0 &+ $1.sizeBytes }
        var bytesDone: UInt64 = 0
        var itemsDone: UInt32 = 0
        let start = Date()

        // Seed with the static estimate so the UI shows a number immediately.
        let seed = estimate(items: request.items)
        progress = TransferProgress(itemsTotal: UInt32(request.items.count), itemsDone: 0,
                                    bytesTotal: bytesTotal, bytesDone: 0,
                                    etaSeconds: seed.seconds, throughputBps: link.sustainedBps)

        for item in request.items {
            do {
                switch request.kind {
                case .load:
                    guard let dest = loadDestination?(item) else { continue }
                    try await data.download(id: item.id, to: dest)
                case .offload:
                    guard let src = offloadSource?(item), let org = origin else { continue }
                    try await data.upload(fileURL: src, origin: org)
                }
            } catch {
                lastError = error.localizedDescription
                return
            }

            bytesDone &+= item.sizeBytes
            itemsDone += 1

            let elapsed = max(Date().timeIntervalSince(start), 0.001)
            let measuredBps = UInt64(Double(bytesDone) / elapsed)
            let remaining = bytesTotal - bytesDone
            let eta = TransferEstimator.eta(bytesRemaining: remaining, measuredBps: max(measuredBps, 1))

            progress = TransferProgress(itemsTotal: UInt32(request.items.count), itemsDone: itemsDone,
                                        bytesTotal: bytesTotal, bytesDone: bytesDone,
                                        etaSeconds: eta, throughputBps: measuredBps)
        }
    }
}

/// Pure trickle decision, mirroring `tucklet-core::trickle`. The app uses this
/// to decide whether to drip a background backup when the charm is near + idle.
public enum Trickle {
    public struct Conditions {
        public var phoneInRange: Bool, phoneIdle: Bool, charging: Bool
        public var batteryPercent: UInt8, pendingItems: UInt32
        public init(phoneInRange: Bool, phoneIdle: Bool, charging: Bool,
                    batteryPercent: UInt8, pendingItems: UInt32) {
            self.phoneInRange = phoneInRange; self.phoneIdle = phoneIdle
            self.charging = charging; self.batteryPercent = batteryPercent
            self.pendingItems = pendingItems
        }
    }
    public struct Decision: Equatable { public let shouldRun: Bool; public let batchSize: UInt32 }
    public static let minBatteryOnBattery: UInt8 = 30

    public static func decide(_ c: Conditions) -> Decision {
        guard c.pendingItems > 0, c.phoneInRange else { return Decision(shouldRun: false, batchSize: 0) }
        let ok = c.charging || (c.phoneIdle && c.batteryPercent >= minBatteryOnBattery)
        guard ok else { return Decision(shouldRun: false, batchSize: 0) }
        let size: UInt32 = c.charging ? 25 : 5
        return Decision(shouldRun: true, batchSize: min(size, c.pendingItems))
    }
}
