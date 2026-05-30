// TrickleScheduler.swift
// Background "trickle" backup: when the charm is near and the phone is idle, a
// small batch of new photos backs up automatically, so the big transfer never
// has to happen. Uses the tested Trickle decision (mirrors tucklet-core).
//
// iOS honesty: background networking is restricted. We register a
// BGProcessingTask (best-effort, system-scheduled) AND run trickle whenever the
// app is foreground/active. Fully-autonomous-while-locked-for-days is an iOS
// limit we don't pretend to beat (Android does this freely; see FINAL_REVIEW).
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
import BackgroundTasks
import UIKit

@MainActor
public final class TrickleScheduler {
    public static let taskId = "app.tucklet.trickle"
    private weak var model: AppModel?

    public init(model: AppModel) {
        self.model = model
    }

    /// Call once at launch (before app finishes launching) to register the task.
    public static func register(handler: @escaping (BGProcessingTask) -> Void) {
        BGTaskScheduler.shared.register(forTaskWithIdentifier: taskId, using: nil) { task in
            handler(task as! BGProcessingTask)
        }
    }

    /// Ask iOS to schedule the next background trickle. The system decides when
    /// (typically while charging + on Wi-Fi), which is exactly when we want it.
    public func scheduleNext() {
        let req = BGProcessingTaskRequest(identifier: Self.taskId)
        req.requiresNetworkConnectivity = true
        req.requiresExternalPower = false
        req.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60)
        try? BGTaskScheduler.shared.submit(req)
    }

    /// Run one trickle pass. Returns how many items it backed up. Safe to call
    /// from the foreground "auto" mode or from the background task handler.
    public func runOnce() async -> Int {
        guard let model, model.trickleEnabled else { return 0 }
        let conditions = model.trickleConditions()
        let decision = Trickle.decide(conditions)
        guard decision.shouldRun else { return 0 }
        return await model.trickleBackup(batchSize: Int(decision.batchSize))
    }

    /// BGProcessingTask handler: run a pass, reschedule, and finish.
    public func handleBackground(_ task: BGProcessingTask) {
        scheduleNext() // always line up the next one
        let work = Task {
            let n = await runOnce()
            task.setTaskCompleted(success: n >= 0)
        }
        task.expirationHandler = { work.cancel() }
    }
}
