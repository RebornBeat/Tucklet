// TuckletGraph.kt
// Process-wide provider of the shared AppRepository, so the foreground UI and
// the background TrickleWorker operate on the same components.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.store

import android.content.Context
import app.tucklet.core.Trickle

object TuckletGraph {
    @Volatile private var repo: AppRepository? = null

    fun repository(context: Context): AppRepository =
        repo ?: synchronized(this) {
            repo ?: AppRepository(context.applicationContext).also { repo = it }
        }

    /**
     * One background trickle pass for WorkManager. Connects if needed, evaluates
     * the (tested) Trickle decision, and drips a small batch. Returns the count
     * moved. Best-effort: returns 0 (not an error) when conditions aren't met.
     */
    suspend fun runTricklePass(context: Context): Int {
        val r = repository(context)
        if (r.connection.value != AppRepository.Conn.CONNECTED) r.connect()
        if (r.connection.value != AppRepository.Conn.CONNECTED) return 0
        if (r.onPhoneItems.value.isEmpty()) r.refreshPhoneLibrary()
        // In the background we treat the phone as idle; charging is unknown here
        // so we let the decision gate on battery instead.
        val decision = Trickle.decide(r.trickleConditions(phoneCharging = false, phoneIdle = true))
        if (!decision.shouldRun) return 0
        return r.trickleBackup(decision.batchSize)
    }
}
