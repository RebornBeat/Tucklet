// TransferEstimator.swift
// Swift mirror of `tucklet-core::estimate` + `::link`. Same formula, same
// numbers, so the time the app shows matches what the device computes.
// (Production option: share the Rust core directly via UniFFI to eliminate
// any drift; this native mirror keeps the app self-contained and is unit-test
// equivalent to the Rust tests.)
//
// License: PolyForm Noncommercial 1.0.0

import Foundation

public struct LinkProfile: Sendable, Equatable {
    public let sustainedBps: UInt64
    public let perFileOverheadMs: UInt32
    public init(_ sustainedBps: UInt64, _ perFileOverheadMs: UInt32) {
        self.sustainedBps = sustainedBps; self.perFileOverheadMs = perFileOverheadMs
    }

    // Conservative ("you will actually get this") profiles from docs/TRANSFER_PERFORMANCE.md.
    public static let c5Wireless     = LinkProfile(9_000_000, 40)
    public static let dualC5Wireless = LinkProfile(15_000_000, 40)
    public static let s3Wireless     = LinkProfile(4_000_000, 50)
    public static let wiredUsbHs     = LinkProfile(30_000_000, 8)

    public static func profile(radio: RadioKind, transport: DataTransport) -> LinkProfile {
        switch transport {
        case .wiredUsbHs: return .wiredUsbHs
        case .softAp, .wifiAware:
            switch radio { case .singleC5: return .c5Wireless; case .dualC5: return .dualC5Wireless }
        }
    }
}

public struct TransferEstimate: Sendable, Equatable {
    public let seconds: UInt32
    public let bytesTotal: UInt64
    public let files: UInt32

    /// "12s", "3 min", "1 hr 4 min".
    public var human: String { TransferEstimator.human(seconds: seconds) }
}

public enum TransferEstimator {
    /// time = sum_over_files( size / sustained + per_file_overhead )
    /// Per-file overhead is why 30 small photos differ from one big file of the
    /// same total size — and why naive total/speed lies to users.
    public static func estimate(sizes: [UInt64], link: LinkProfile) -> TransferEstimate {
        let files = UInt32(sizes.count)
        let bytesTotal = sizes.reduce(UInt64(0), &+)
        let bps = UInt128(max(link.sustainedBps, 1))
        var ms: UInt128 = 0
        for s in sizes {
            ms += (UInt128(s) * 1000) / bps
            ms += UInt128(link.perFileOverheadMs)
        }
        let seconds = UInt64((ms + 999) / 1000)
        return TransferEstimate(seconds: UInt32(min(seconds, UInt64(UInt32.max))),
                                bytesTotal: bytesTotal, files: files)
    }

    /// Live ETA from measured throughput, so the displayed number stays honest.
    public static func eta(bytesRemaining: UInt64, measuredBps: UInt64) -> UInt32 {
        guard measuredBps > 0 else { return UInt32.max }
        let s = (bytesRemaining + measuredBps - 1) / measuredBps
        return UInt32(min(s, UInt64(UInt32.max)))
    }

    public static func human(seconds total: UInt32) -> String {
        if total < 60 { return "\(total)s" }
        if total < 3600 {
            let m = total / 60, s = total % 60
            return s == 0 ? "\(m) min" : "\(m) min \(s)s"
        }
        let h = total / 3600, m = (total % 3600) / 60
        return m == 0 ? "\(h) hr" : "\(h) hr \(m) min"
    }
}

// UInt128 is available in Swift 6 stdlib. If targeting an older toolchain,
// replace the UInt128 arithmetic above with Double math (acceptable for an
// estimate) — CONFIRM availability against your Swift version.
