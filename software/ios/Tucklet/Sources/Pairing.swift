// Pairing.swift
// First-run pairing via AccessorySetupKit (iOS 18+): "bring close, tap to pair."
// After this one-time step the charm is invisible — silent BLE reconnect +
// cryptographic challenge-response (see BLEControlClient / docs FINAL_REVIEW §3).
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
import SwiftUI
import AccessorySetupKit
import CoreBluetooth

/// Wraps an ASAccessorySession to present the system pairing sheet for a
/// Tucklet, identified by its BLE service UUID.
///
/// CONFIRM against the iOS 26 AccessorySetupKit SDK: the exact
/// ASDiscoveryDescriptor property names and the picker-completion shape have
/// evolved; the structure below reflects the iOS 18 ASK model. The full
/// AirPods-grade proximity parity is EU-only at present (DMA) — the base
/// tap-to-pair flow is worldwide.
@MainActor
public final class PairingManager: ObservableObject {
    @Published public private(set) var isPaired = false
    @Published public private(set) var lastError: String?

    private let session = ASAccessorySession()
    // Must match the firmware GATT service UUID (ble.rs SVC_UUID).
    private let serviceUUID = CBUUID(string: "F0CC0001-0000-1000-8000-00805F9B34FB")

    public init() {
        session.activate(on: .main) { [weak self] event in
            self?.handle(event)
        }
    }

    /// Present the system "bring your Tucklet close and tap" sheet.
    public func showPicker() {
        let descriptor = ASDiscoveryDescriptor()
        descriptor.bluetoothServiceUUID = serviceUUID
        let item = ASPickerDisplayItem(
            name: "Tucklet",
            productImage: UIImage(named: "TuckletGlyph") ?? UIImage(),
            descriptor: descriptor
        )
        session.showPicker(for: [item]) { [weak self] error in
            if let error { self?.lastError = error.localizedDescription }
        }
    }

    private func handle(_ event: ASAccessoryEvent) {
        switch event.eventType {
        case .accessoryAdded:
            // The system has paired the accessory at the OS level. The app then
            // performs the cryptographic enrollment over BLE (PairRequest ->
            // device returns its pubkey), which is what makes later connects
            // silent and trustworthy.
            isPaired = true
        case .accessoryRemoved:
            isPaired = false
        default:
            break
        }
    }

    /// "Forget this Tucklet" — remove the accessory from the system and (the
    /// caller also) revoke trust on the device + locally.
    public func forget(_ accessory: ASAccessory) {
        session.removeAccessory(accessory) { [weak self] error in
            if let error { self?.lastError = error.localizedDescription }
            else { self?.isPaired = false }
        }
    }
}

/// First-run / not-paired screen. Calm, single clear action.
struct OnboardingView: View {
    @EnvironmentObject var model: AppModel
    @StateObject private var pairing = PairingManager()

    var body: some View {
        VStack(spacing: 24) {
            Spacer()
            Image(systemName: "circle.hexagongrid.fill")
                .font(.system(size: 64)).foregroundStyle(Brand.accent)
            Text("Meet your Tucklet")
                .font(.system(size: 28, weight: .bold)).foregroundStyle(Brand.ink)
            Text("Bring it close and tap to connect. You'll only do this once — after that it just works whenever it's near.")
                .multilineTextAlignment(.center)
                .foregroundStyle(Brand.muted)
                .padding(.horizontal, 32)
            Spacer()
            Button {
                pairing.showPicker()
            } label: {
                Text("Connect my Tucklet")
                    .font(.headline).frame(maxWidth: .infinity).padding()
                    .background(Brand.accent).foregroundStyle(.white)
                    .clipShape(RoundedRectangle(cornerRadius: 14))
            }
            .padding(.horizontal, 24)
            if let err = pairing.lastError {
                Text(err).font(.caption).foregroundStyle(.red)
            }
            Spacer().frame(height: 12)
        }
        .background(Brand.paper.ignoresSafeArea())
        .onChange(of: pairing.isPaired) { _, paired in
            if paired {
                Task {
                    // OS-level pairing done; now do the cryptographic enrollment
                    // over BLE and connect.
                    await model.completeEnrollmentAndConnect()
                }
            }
        }
    }
}
