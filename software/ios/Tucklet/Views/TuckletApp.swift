// TuckletApp.swift
// App entry. Warm, calm, keepsake feel — see docs/UX_SPEC.md.
// License: PolyForm Noncommercial 1.0.0

import SwiftUI
import BackgroundTasks

@main
struct TuckletApp: App {
    @StateObject private var model = AppModel()

    init() {
        // Register the background trickle task before launch finishes. The
        // handler is wired to the model's scheduler when the task fires.
        TrickleScheduler.register { task in
            Task { @MainActor in
                let scheduler = TrickleScheduler(model: AppModelHolder.shared ?? AppModel())
                scheduler.handleBackground(task)
            }
        }
    }

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(model)
                .tint(Brand.accent)
                .onAppear { AppModelHolder.shared = model }
        }
    }
}

/// Bridges the @main App's @StateObject to the background-task closure, which is
/// registered before the model exists in the view tree.
@MainActor enum AppModelHolder { static var shared: AppModel? }

enum Brand {
    static let accent = Color(red: 0.71, green: 0.40, blue: 0.31)   // dusty terracotta
    static let ink    = Color(red: 0.23, green: 0.17, blue: 0.14)
    static let paper  = Color(red: 0.98, green: 0.97, blue: 0.95)
    static let muted  = Color(red: 0.55, green: 0.46, blue: 0.39)
}

struct RootView: View {
    @EnvironmentObject var model: AppModel
    var body: some View {
        Group {
            if model.isPaired {
                TabView {
                    HomeView()
                        .tabItem { Label("Home", systemImage: "heart.circle") }
                    LibraryView()
                        .tabItem { Label("Library", systemImage: "square.grid.2x2") }
                    SettingsView()
                        .tabItem { Label("Settings", systemImage: "gearshape") }
                }
                .task { await model.connect() }
            } else {
                OnboardingView()
            }
        }
    }
}
