// SessionManager.kt
// Orchestrates opening a data session: request it over BLE (with the signed
// challenge), join the charm's Wi-Fi, and hand back a DataClient pinned to it.
// Mirrors tucklet-core::variant transport ordering.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.net

import app.tucklet.ble.BleControlClient
import app.tucklet.protocol.*
import app.tucklet.wifi.WifiBinding
import app.tucklet.wifi.WifiConnector

class SessionManager(
    private val ble: BleControlClient,
    private val wifi: WifiConnector,
) {
    private var binding: WifiBinding? = null

    /** Conservative transport ordering: Aware first (seamless), then SoftAP. We
     *  never auto-pick the wired path for a wireless session. */
    fun usableTransports(caps: DeviceCapabilities): List<DataTransport> {
        val order = listOf(DataTransport.WIFI_AWARE, DataTransport.SOFT_AP)
        return order.filter { caps.supports(it) }
    }

    /**
     * Open a data session and return a connected DataClient.
     * On Android both Aware and SoftAP resolve to a WifiNetworkSpecifier join
     * (the grant carries the SSID/service + PSK either way).
     */
    suspend fun openDataSession(caps: DeviceCapabilities, challengeSignature: String): DataClient {
        val transport = usableTransports(caps).firstOrNull() ?: DataTransport.SOFT_AP
        val grant: SessionGrant = ble.openSession(SessionRequest(challengeSignature, transport))
        val b = wifi.join(grant.ssidOrService, grant.psk)
        binding = b
        return DataClient(b.network, grant.ip, grant.token)
    }

    /** End the session: release the Wi-Fi binding (the charm tears down the AP
     *  on its side when the session times out / disconnects). */
    fun close() {
        binding?.release(); binding = null
    }
}
