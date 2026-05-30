// TransferEstimator.kt
// Pure Kotlin mirror of `tucklet-core` (link profiles + estimator + trickle).
// Same constants and formulas as the Rust, which is the tested source of truth.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.core

import app.tucklet.protocol.DataTransport
import app.tucklet.protocol.RadioKind
import kotlin.math.min

/** Throughput model: sustained bytes/sec + a fixed per-file overhead (ms). */
data class LinkProfile(val sustainedBps: Long, val perFileOverheadMs: Int) {
    companion object {
        // Conservative, measured numbers (docs/TRANSFER_PERFORMANCE.md).
        val C5_WIRELESS = LinkProfile(9_000_000, 40)
        val DUAL_C5_WIRELESS = LinkProfile(15_000_000, 40)
        val WIRED_USB_HS = LinkProfile(30_000_000, 8)

        fun profileFor(radio: RadioKind, transport: DataTransport): LinkProfile =
            when (transport) {
                DataTransport.WIRED_USB_HS -> WIRED_USB_HS
                DataTransport.SOFT_AP, DataTransport.WIFI_AWARE -> when (radio) {
                    RadioKind.SINGLE_C5 -> C5_WIRELESS
                    RadioKind.DUAL_C5 -> DUAL_C5_WIRELESS
                }
            }
    }
}

data class Estimate(val seconds: Int, val bytesTotal: Long, val files: Int) {
    val human: String get() = TransferEstimator.humanSeconds(seconds)
}

object TransferEstimator {
    /**
     * time = sum_over_files( size / sustained + per_file_overhead )
     *
     * The per-file overhead is what makes "30 small photos" differ from one big
     * video of the same total size — exactly why `total / speed` lies.
     */
    fun estimate(sizes: List<Long>, link: LinkProfile): Estimate {
        val bps = link.sustainedBps.coerceAtLeast(1)
        var ms = 0L
        for (s in sizes) {
            ms += (s * 1000L) / bps
            ms += link.perFileOverheadMs.toLong()
        }
        val seconds = (ms + 999) / 1000 // round up
        return Estimate(
            seconds = seconds.coerceAtMost(Int.MAX_VALUE.toLong()).toInt(),
            bytesTotal = sizes.sum(),
            files = sizes.size,
        )
    }

    /** Recompute remaining seconds mid-transfer from measured throughput. */
    fun eta(bytesRemaining: Long, measuredBps: Long): Int {
        if (measuredBps <= 0) return Int.MAX_VALUE
        val s = (bytesRemaining + measuredBps - 1) / measuredBps // ceil
        return s.coerceAtMost(Int.MAX_VALUE.toLong()).toInt()
    }

    fun humanSeconds(total: Int): String = when {
        total < 60 -> "${total}s"
        total < 3600 -> {
            val m = total / 60; val s = total % 60
            if (s == 0) "$m min" else "$m min ${s}s"
        }
        else -> {
            val h = total / 3600; val m = (total % 3600) / 60
            if (m == 0) "$h hr" else "$h hr $m min"
        }
    }
}

/** Trickle: solve slow bandwidth with time, not speed — drip new items whenever
 *  it's cheap, so the big transfer never has to happen. */
object Trickle {
    const val MIN_BATTERY_ON_BATTERY = 30

    data class Conditions(
        val phoneInRange: Boolean,
        val phoneIdle: Boolean,
        val charging: Boolean,
        val batteryPercent: Int,
        val pendingItems: Int,
    )

    data class Decision(val shouldRun: Boolean, val batchSize: Int)

    fun decide(c: Conditions): Decision {
        if (c.pendingItems == 0 || !c.phoneInRange) return Decision(false, 0)
        val ok = c.charging || (c.phoneIdle && c.batteryPercent >= MIN_BATTERY_ON_BATTERY)
        if (!ok) return Decision(false, 0)
        val batch = if (c.charging) 25 else 5
        return Decision(true, min(batch, c.pendingItems))
    }
}
