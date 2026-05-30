// TrickleWorker.kt
// Real background backup on Android (unlike iOS's tight limits): a periodic
// WorkManager job that, when the charm is reachable and conditions are good,
// drips a small batch of new photos to it. Uses the tested Trickle decision.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.trickle

import android.content.Context
import androidx.work.*
import app.tucklet.store.TuckletGraph
import java.util.concurrent.TimeUnit

class TrickleWorker(
    appContext: Context,
    params: WorkerParameters,
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        // The work graph holds the long-lived components (built lazily). It runs
        // one trickle pass: connect if needed, evaluate Trickle.decide, drip.
        val moved = runCatching { TuckletGraph.runTricklePass(applicationContext) }
            .getOrElse { return Result.retry() }
        return Result.success(workDataOf("moved" to moved))
    }

    companion object {
        private const val WORK_NAME = "tucklet.trickle"

        /** Enqueue the periodic trickle (every ~3h, only when not low battery).
         *  WorkManager coalesces and respects Doze; this is genuinely allowed to
         *  run in the background, which is the Android advantage over iOS. */
        fun schedule(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiresBatteryNotLow(true)
                .build()
            val request = PeriodicWorkRequestBuilder<TrickleWorker>(3, TimeUnit.HOURS)
                .setConstraints(constraints)
                .build()
            WorkManager.getInstance(context)
                .enqueueUniquePeriodicWork(WORK_NAME, ExistingPeriodicWorkPolicy.KEEP, request)
        }

        fun cancel(context: Context) {
            WorkManager.getInstance(context).cancelUniqueWork(WORK_NAME)
        }
    }
}
