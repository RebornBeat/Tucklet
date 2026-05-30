// WifiConnector.kt
// Programmatically join the charm's one-time SoftAP and bind to that Network,
// so the HTTP traffic goes over the charm link (not the phone's main Wi-Fi).
// This is the seamless path Android allows that iOS does not: no trip to
// Settings, just a one-time system approval the first time.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.wifi

import android.content.Context
import android.net.*
import android.net.wifi.WifiNetworkSpecifier
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.withTimeout

/** A live binding to the charm's Wi-Fi network. Release it when the session ends. */
class WifiBinding(
    val network: Network,
    private val cm: ConnectivityManager,
    private val callback: ConnectivityManager.NetworkCallback,
) {
    fun release() {
        runCatching { cm.unregisterNetworkCallback(callback) }
    }
}

class WifiConnector(context: Context) {
    private val cm = context.applicationContext
        .getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    /**
     * Join the SoftAP described by the SessionGrant and return a binding to the
     * resulting Network. The OkHttp client uses `network.socketFactory` so the
     * request is pinned to the charm, leaving the phone's normal data path
     * untouched.
     *
     * CONFIRM: Android shows a one-time "connect to <SSID>?" approval for a
     * WifiNetworkSpecifier; once approved the system may auto-approve later
     * one-time SSIDs from the same app. The charm uses a fresh SSID per session,
     * so the binding (not a saved network) is the right primitive.
     */
    suspend fun join(ssid: String, psk: String, timeoutMs: Long = 20_000): WifiBinding {
        val specifier = WifiNetworkSpecifier.Builder()
            .setSsid(ssid)
            .setWpa2Passphrase(psk)
            .build()

        val request = NetworkRequest.Builder()
            .addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
            // The charm AP has no internet; don't let the system drop it for that.
            .removeCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .setNetworkSpecifier(specifier)
            .build()

        val available = CompletableDeferred<Network>()
        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) { available.complete(network) }
            override fun onUnavailable() {
                if (!available.isCompleted) available.completeExceptionally(IllegalStateException("Wi-Fi join unavailable"))
            }
        }
        cm.requestNetwork(request, callback)
        val network = try {
            withTimeout(timeoutMs) { available.await() }
        } catch (t: Throwable) {
            runCatching { cm.unregisterNetworkCallback(callback) }
            throw t
        }
        return WifiBinding(network, cm, callback)
    }
}
