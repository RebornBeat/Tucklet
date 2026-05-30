// AppViewModel.kt
// Thin AndroidViewModel that exposes the shared repository's flows to Compose
// and runs UI actions on the viewModelScope.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.store

import android.app.Application
import android.graphics.Bitmap
import android.os.BatteryManager
import android.content.Context
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import app.tucklet.core.Estimate
import app.tucklet.core.TransferEstimator
import app.tucklet.protocol.*
import app.tucklet.trickle.TrickleWorker
import kotlinx.coroutines.launch

class AppViewModel(app: Application) : AndroidViewModel(app) {
    val repo = TuckletGraph.repository(app)

    // Re-expose flows for Compose collectAsState.
    val connection get() = repo.connection
    val capabilities get() = repo.capabilities
    val status get() = repo.status
    val manifest get() = repo.manifest
    val onPhoneItems get() = repo.onPhoneItems
    val isPaired get() = repo.isPaired
    val lastOffloadIds get() = repo.lastOffloadIds
    val errorText get() = repo.errorText

    fun libraryGroups() = repo.libraryGroups()
    fun pendingBackupCount() = repo.pendingBackupCount()

    fun connect() = viewModelScope.launch { repo.connect() }
    fun enrollAndConnect(mac: String) = viewModelScope.launch { repo.enrollAndConnect(mac) }
    fun loadLibrary() = viewModelScope.launch { repo.loadLibrary() }
    fun refreshPhoneLibrary() = viewModelScope.launch { repo.refreshPhoneLibrary() }

    suspend fun thumbnail(item: MediaItem): Bitmap? = repo.thumbnail(item)

    fun estimate(items: List<MediaItem>): Estimate =
        TransferEstimator.estimate(items.map { it.sizeBytes }, repo.linkProfile())

    /** Run an offload batch (export + upload), then delete originals + record undo. */
    fun runOffload(items: List<MediaItem>, onDone: () -> Unit) = viewModelScope.launch {
        val moved = ArrayList<String>()
        for (item in items) {
            runCatching { repo.offloadItem(item) }.onSuccess { moved += item.id }.onFailure { return@launch }
        }
        repo.finishOffload(moved)
        onDone()
    }

    fun runLoad(items: List<MediaItem>, policy: TemporaryPolicy, onDone: () -> Unit) = viewModelScope.launch {
        for (item in items) runCatching { repo.loadItem(item, policy) }
        onDone()
    }

    fun restore(item: MediaItem) = viewModelScope.launch { runCatching { repo.restore(item) } }
    fun delete(item: MediaItem) = viewModelScope.launch { repo.delete(item) }
    fun undoLastOffload() = viewModelScope.launch { repo.undoLastOffload() }
    fun forget() = viewModelScope.launch { repo.forget() }

    /** Foreground "auto" trickle: enable the periodic worker and run one pass now. */
    fun enableTrickle(context: Context) {
        TrickleWorker.schedule(context)
        viewModelScope.launch {
            val charging = isCharging(context)
            val cond = repo.trickleConditions(phoneCharging = charging, phoneIdle = true)
            val d = app.tucklet.core.Trickle.decide(cond)
            if (d.shouldRun) repo.trickleBackup(d.batchSize)
        }
    }

    fun disableTrickle(context: Context) = TrickleWorker.cancel(context)

    private fun isCharging(context: Context): Boolean {
        val bm = context.getSystemService(Context.BATTERY_SERVICE) as? BatteryManager
        return bm?.isCharging ?: false
    }

    fun byteFormat(bytes: Long): String {
        val units = arrayOf("B", "KB", "MB", "GB", "TB")
        var v = bytes.toDouble(); var i = 0
        while (v >= 1024 && i < units.size - 1) { v /= 1024; i++ }
        return if (i == 0) "${v.toInt()} ${units[i]}" else String.format("%.1f %s", v, units[i])
    }
}
