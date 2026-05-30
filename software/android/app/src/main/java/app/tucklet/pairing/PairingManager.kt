// PairingManager.kt
// First-run pairing via CompanionDeviceManager: the system shows a chooser of
// nearby Tucklets (filtered by our BLE service UUID), the user taps one, and the
// association is kept so the app can reconnect + use BLE in the background. The
// cryptographic enrollment then happens over BLE (PairRequest).
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.pairing

import android.bluetooth.le.ScanFilter
import android.companion.AssociationRequest
import android.companion.BluetoothLeDeviceFilter
import android.companion.CompanionDeviceManager
import android.content.Context
import android.content.IntentSender
import android.os.ParcelUuid
import app.tucklet.ble.TuckletGatt
import java.util.concurrent.Executor

class PairingManager(context: Context) {
    private val cdm = context.getSystemService(Context.COMPANION_DEVICE_SERVICE) as CompanionDeviceManager

    /** Existing associations (MAC strings) so a returning user skips onboarding. */
    val associations: List<String> get() = runCatching { cdm.associations }.getOrDefault(emptyList())
    val isAssociated: Boolean get() = associations.isNotEmpty()

    /**
     * Begin association. The caller launches the returned IntentSender with an
     * ActivityResult launcher; on success the selected device is delivered back
     * and the caller connects + enrolls over BLE.
     *
     * CONFIRM: on API 33+ prefer the callback-based associate(...) overload; the
     * IntentSender path below is the widely-supported one.
     */
    fun associate(
        executor: Executor,
        onReady: (IntentSender) -> Unit,
        onFailure: (CharSequence?) -> Unit,
    ) {
        val deviceFilter = BluetoothLeDeviceFilter.Builder()
            .setScanFilter(
                ScanFilter.Builder()
                    .setServiceUuid(ParcelUuid(TuckletGatt.SERVICE))
                    .build()
            )
            .build()

        val request = AssociationRequest.Builder()
            .addDeviceFilter(deviceFilter)
            .setSingleDevice(false)
            .build()

        cdm.associate(request, executor, object : CompanionDeviceManager.Callback() {
            override fun onAssociationPending(intentSender: IntentSender) = onReady(intentSender)
            override fun onAssociationCreated(associationInfo: android.companion.AssociationInfo) { /* delivered via result too */ }
            override fun onFailure(error: CharSequence?) = onFailure(error)
        })
    }

    /** Forget this Tucklet on this phone (also revoke locally / disconnect). */
    fun disassociate(macAddress: String) {
        runCatching {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                val id = cdm.myAssociations.firstOrNull { it.deviceMacAddress?.toString().equals(macAddress, true) }?.id
                if (id != null) cdm.disassociate(id)
            } else {
                @Suppress("DEPRECATION") cdm.disassociate(macAddress)
            }
        }
    }
}
