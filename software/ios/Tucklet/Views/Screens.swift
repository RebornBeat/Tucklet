// Screens.swift
// Home / Library / Transfer / Settings, plus the plain-language state badge.
// License: PolyForm Noncommercial 1.0.0

import SwiftUI

// MARK: - Plain-language state badge (never the word "cache")

struct StateBadge: View {
    let state: ItemState
    var body: some View {
        let (icon, text) = display
        Label(text, systemImage: icon)
            .font(.caption2)
            .padding(.horizontal, 6).padding(.vertical, 3)
            .background(Capsule().fill(Brand.accent.opacity(0.12)))
            .foregroundStyle(Brand.ink)
            .accessibilityLabel(text)
    }
    private var display: (String, String) {
        switch state {
        case .onPhone: return ("iphone", "On phone")
        case .onTucklet: return ("circle.hexagongrid", "On Tucklet")
        case .temporary(let exp):
            if let exp { return ("clock", "Temporary · \(Self.remaining(exp))") }
            return ("clock", "Temporary")
        }
    }
    private static func remaining(_ epoch: Int64) -> String {
        let secs = max(epoch - Int64(Date().timeIntervalSince1970), 0)
        return TransferEstimator.human(seconds: UInt32(min(secs, Int64(UInt32.max))))
    }
}

// MARK: - Home

struct HomeView: View {
    @EnvironmentObject var model: AppModel
    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 20) {
                    presenceCard
                    summaryCard
                    quickActions
                }.padding()
            }
            .background(Brand.paper.ignoresSafeArea())
            .navigationTitle("Tucklet")
        }
    }

    private var presenceCard: some View {
        VStack(spacing: 8) {
            Image(systemName: model.connectionState == .connected ? "circle.hexagongrid.fill" : "circle.hexagongrid")
                .font(.system(size: 44)).foregroundStyle(Brand.accent)
            switch model.connectionState {
            case .connected:
                if let s = model.status {
                    Text("\(s.batteryPercent)%\(s.charging ? " · charging" : "")")
                        .font(.headline).foregroundStyle(Brand.ink)
                }
                Text(model.freeText).font(.subheadline).foregroundStyle(Brand.muted)
                Text(model.storageDescription).font(.caption).foregroundStyle(Brand.muted)
            case .connecting: ProgressView("Finding your Tucklet…")
            case .idle: Text("Bring your Tucklet near").foregroundStyle(Brand.muted)
            case .error(let e): Text(e).font(.caption).foregroundStyle(.red)
            }
        }
        .frame(maxWidth: .infinity).padding(24)
        .background(RoundedRectangle(cornerRadius: 20).fill(.white))
    }

    private var summaryCard: some View {
        let pending = model.manifest?.items.filter { $0.state == .onPhone }.count ?? 0
        return HStack {
            Image(systemName: pending == 0 ? "checkmark.seal.fill" : "tray.and.arrow.up")
                .foregroundStyle(Brand.accent)
            Text(pending == 0 ? "Everything's backed up" : "\(pending) photos waiting to back up")
                .foregroundStyle(Brand.ink)
            Spacer()
        }
        .padding().background(RoundedRectangle(cornerRadius: 16).fill(.white))
    }

    private var quickActions: some View {
        VStack(spacing: 12) {
            NavigationLink { LibraryView() } label: {
                actionRow("Free up space", "tray.and.arrow.up", "Move photos to your Tucklet")
            }
            NavigationLink { LibraryView() } label: {
                actionRow("Get photos", "tray.and.arrow.down", "Bring a copy back to your phone")
            }
        }
    }
    private func actionRow(_ title: String, _ icon: String, _ sub: String) -> some View {
        HStack(spacing: 14) {
            Image(systemName: icon).font(.title3).foregroundStyle(Brand.accent).frame(width: 28)
            VStack(alignment: .leading) {
                Text(title).font(.headline).foregroundStyle(Brand.ink)
                Text(sub).font(.caption).foregroundStyle(Brand.muted)
            }
            Spacer(); Image(systemName: "chevron.right").foregroundStyle(Brand.muted)
        }
        .padding().background(RoundedRectangle(cornerRadius: 16).fill(.white))
    }
}

// MARK: - Library (per-app organization; metadata/thumbnails only)

struct LibraryView: View {
    @EnvironmentObject var model: AppModel
    @State private var selection = Set<String>()
    @State private var showTransfer = false
    @State private var kind: TransferKind = .offload

    var grouped: [(app: String, items: [MediaItem])] {
        let items = model.manifest?.items ?? []
        let groups = Dictionary(grouping: items, by: { $0.origin.app })
        return groups.keys.sorted().map { (app: $0, items: groups[$0]!) }
    }

    var body: some View {
        NavigationStack {
            List {
                ForEach(grouped, id: \.app) { group in
                    Section(group.app) {
                        ForEach(group.items) { item in row(item) }
                    }
                }
            }
            .background(Brand.paper)
            .navigationTitle("Library")
            .toolbar {
                if !selection.isEmpty {
                    ToolbarItemGroup(placement: .bottomBar) {
                        Button { kind = .offload; showTransfer = true } label: { Label("Free up space", systemImage: "tray.and.arrow.up") }
                        Spacer()
                        Button { kind = .load; showTransfer = true } label: { Label("Get a copy", systemImage: "tray.and.arrow.down") }
                    }
                }
            }
            .task { if model.manifest == nil { await model.loadLibrary() } }
            .sheet(isPresented: $showTransfer) {
                TransferSheet(kind: kind, items: selectedItems)
            }
        }
    }

    private var selectedItems: [MediaItem] {
        (model.manifest?.items ?? []).filter { selection.contains($0.id) }
    }

    private func row(_ item: MediaItem) -> some View {
        HStack {
            Image(systemName: selection.contains(item.id) ? "checkmark.circle.fill" : "circle")
                .foregroundStyle(selection.contains(item.id) ? Brand.accent : Brand.muted)
            VStack(alignment: .leading, spacing: 4) {
                Text(item.name).foregroundStyle(Brand.ink).lineLimit(1)
                HStack { StateBadge(state: item.state); Text(ByteFormat.short(item.sizeBytes)).font(.caption2).foregroundStyle(Brand.muted) }
            }
            Spacer()
            Image(systemName: item.isVideo ? "video" : "photo").foregroundStyle(Brand.muted)
        }
        .contentShape(Rectangle())
        .onTapGesture {
            if selection.contains(item.id) { selection.remove(item.id) } else { selection.insert(item.id) }
        }
    }
}

// MARK: - Transfer sheet (the most important screen: shows the time up front)

struct TransferSheet: View {
    @EnvironmentObject var model: AppModel
    @Environment(\.dismiss) var dismiss
    let kind: TransferKind
    let items: [MediaItem]

    @State private var mode: TransferMode = .batch
    @State private var policy: TemporaryPolicy = .oneWeek
    @StateObject private var engine = TransferEngineHolder()

    private var transferItems: [TransferItem] {
        items.map { TransferItem(id: $0.id, sizeBytes: $0.sizeBytes, mime: $0.mime) }
    }
    private var estimate: TransferEstimate {
        TransferEstimator.estimate(sizes: items.map(\.sizeBytes), link: model.currentLinkProfile())
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 22) {
                VStack(spacing: 6) {
                    Text(kind == .offload ? "Free up space" : "Get a copy").font(.title2.bold()).foregroundStyle(Brand.ink)
                    Text("\(items.count) items · \(ByteFormat.short(estimate.bytesTotal))").foregroundStyle(Brand.muted)
                }
                // The headline number: estimated time, before anything moves.
                VStack(spacing: 4) {
                    Text("About \(estimate.human)").font(.system(size: 34, weight: .bold)).foregroundStyle(Brand.accent)
                    Text("over \(transportLabel)").font(.caption).foregroundStyle(Brand.muted)
                }
                .padding().frame(maxWidth: .infinity)
                .background(RoundedRectangle(cornerRadius: 18).fill(.white))

                Picker("When", selection: $mode) {
                    Text("Now").tag(TransferMode.batch)
                    Text("Automatically").tag(TransferMode.trickle)
                }.pickerStyle(.segmented)

                if kind == .load {
                    HStack {
                        Text("Keep on phone for").foregroundStyle(Brand.ink); Spacer()
                        Picker("", selection: $policy) {
                            ForEach(TemporaryPolicy.allCases, id: \.self) { Text($0.label).tag($0) }
                        }.tint(Brand.accent)
                    }.padding().background(RoundedRectangle(cornerRadius: 14).fill(.white))
                }

                if let p = engine.engine?.progress, engine.engine?.isRunning == true {
                    VStack(spacing: 8) {
                        ProgressView(value: Double(p.bytesDone), total: Double(max(p.bytesTotal, 1)))
                        Text("\(p.itemsDone) of \(p.itemsTotal) · \(TransferEstimator.human(seconds: p.etaSeconds)) left")
                            .font(.caption).foregroundStyle(Brand.muted)
                    }
                }

                Spacer()
                Button {
                    Task { await start() }
                } label: {
                    Text(mode == .batch ? "Start" : "Turn on auto backup")
                        .font(.headline).frame(maxWidth: .infinity).padding()
                        .background(Brand.accent).foregroundStyle(.white)
                        .clipShape(RoundedRectangle(cornerRadius: 14))
                }
            }
            .padding()
            .background(Brand.paper.ignoresSafeArea())
            .navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Close") { dismiss() } } }
        }
    }

    private var transportLabel: String {
        switch model.capabilities?.bestWireless {
        case .wifiAware: return "Wi-Fi"
        case .softAp: return "Wi-Fi"
        default: return "Wi-Fi"
        }
    }

    private func start() async {
        guard let e = model.makeTransferEngine() else { return }
        engine.engine = e
        let req = TransferRequest(kind: kind, mode: mode, items: transferItems,
                                  temporaryPolicy: kind == .load ? policy : nil)
        if mode == .batch {
            await e.runBatch(req,
                             loadDestination: { item in
                                 FileManager.default.temporaryDirectory.appendingPathComponent(item.id)
                             })
        }
        // Trickle mode hands off to the background scheduler (see TransferEngine.Trickle).
        if e.lastError == nil { dismiss() }
    }
}

/// Small holder so the sheet can own an engine created after model wiring.
@MainActor final class TransferEngineHolder: ObservableObject {
    @Published var engine: TransferEngine?
}

// MARK: - Settings

struct SettingsView: View {
    @EnvironmentObject var model: AppModel
    var body: some View {
        NavigationStack {
            Form {
                Section("Backup") {
                    Toggle("Back up automatically", isOn: $model.trickleEnabled)
                    Toggle("Only while charging", isOn: $model.trickleOnlyWhileCharging)
                        .disabled(!model.trickleEnabled)
                }
                Section("Temporary copies") {
                    Picker("Default keep time", selection: $model.defaultTemporaryPolicy) {
                        ForEach(TemporaryPolicy.allCases, id: \.self) { Text($0.label).tag($0) }
                    }
                }
                Section("Device") {
                    LabeledContent("Storage", value: model.storageDescription)
                    if let s = model.status {
                        LabeledContent("Battery", value: "\(s.batteryPercent)%")
                        LabeledContent("Firmware", value: s.firmwareVersion)
                    }
                }
                Section("Paired phones") {
                    // Production: list allow-list entries from the device; allow "Forget".
                    Text("This phone").foregroundStyle(Brand.ink)
                    Button("Forget this Tucklet", role: .destructive) { /* revoke flow */ }
                }
            }
            .navigationTitle("Settings")
        }
    }
}
