// TuckletApp.swift
// App entry. Warm, calm, keepsake feel — see docs/UX_SPEC.md.
// License: PolyForm Noncommercial 1.0.0

import SwiftUI

@main
struct TuckletApp: App {
    @StateObject private var model = AppModel()
    var body: some Scene {
        WindowGroup {
            RootView().environmentObject(model)
                .tint(Brand.accent)
        }
    }
}

enum Brand {
    static let accent = Color(red: 0.71, green: 0.40, blue: 0.31)   // dusty terracotta
    static let ink    = Color(red: 0.23, green: 0.17, blue: 0.14)
    static let paper  = Color(red: 0.98, green: 0.97, blue: 0.95)
    static let muted  = Color(red: 0.55, green: 0.46, blue: 0.39)
}

struct RootView: View {
    @EnvironmentObject var model: AppModel
    var body: some View {
        TabView {
            HomeView()
                .tabItem { Label("Home", systemImage: "heart.circle") }
            LibraryView()
                .tabItem { Label("Library", systemImage: "square.grid.2x2") }
            SettingsView()
                .tabItem { Label("Settings", systemImage: "gearshape") }
        }
        .task { await model.connect() }
    }
}
