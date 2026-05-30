// MainActivity.kt
// Hosts the Compose app, requests the runtime permissions, and drives the
// CompanionDeviceManager pairing chooser (the one-tap onboarding).
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.ui

import android.Manifest
import android.bluetooth.BluetoothDevice
import android.bluetooth.le.ScanResult
import android.companion.AssociationInfo
import android.companion.CompanionDeviceManager
import android.content.Intent
import android.content.IntentSender
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.IntentSenderRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.core.content.ContextCompat
import app.tucklet.store.AppViewModel

class MainActivity : ComponentActivity() {
    private val vm: AppViewModel by viewModels()

    private val permissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { /* proceed regardless; features degrade gracefully if denied */ }

    private val associationLauncher = registerForActivityResult(
        ActivityResultContracts.StartIntentSenderForResult()
    ) { result ->
        val mac = extractMac(result.data)
        if (mac != null) vm.enrollAndConnect(mac)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestRuntimePermissions()
        setContent {
            TuckletTheme {
                TuckletApp(vm = vm, onStartPairing = { startPairing() })
            }
        }
        // Try a silent reconnect for already-associated users.
        if (vm.isPaired.value) vm.connect()
    }

    private fun requestRuntimePermissions() {
        val perms = buildList {
            add(Manifest.permission.BLUETOOTH_SCAN)
            add(Manifest.permission.BLUETOOTH_CONNECT)
            if (Build.VERSION.SDK_INT >= 33) {
                add(Manifest.permission.READ_MEDIA_IMAGES)
                add(Manifest.permission.READ_MEDIA_VIDEO)
                add(Manifest.permission.POST_NOTIFICATIONS)
            } else {
                add(Manifest.permission.READ_EXTERNAL_STORAGE)
            }
        }.filter {
            ContextCompat.checkSelfPermission(this, it) != android.content.pm.PackageManager.PERMISSION_GRANTED
        }
        if (perms.isNotEmpty()) permissionLauncher.launch(perms.toTypedArray())
    }

    private fun startPairing() {
        vm.repo.pairing.associate(
            executor = ContextCompat.getMainExecutor(this),
            onReady = { sender: IntentSender ->
                associationLauncher.launch(IntentSenderRequest.Builder(sender).build())
            },
            onFailure = { /* surface via vm.errorText in a fuller build */ },
        )
    }

    /** Pull the device MAC from the association result, across API levels.
     *  CONFIRM the exact extra keys against your target SDK. */
    private fun extractMac(data: Intent?): String? {
        data ?: return null
        if (Build.VERSION.SDK_INT >= 34) {
            val assoc = data.getParcelableExtra(
                CompanionDeviceManager.EXTRA_ASSOCIATION, AssociationInfo::class.java
            )
            assoc?.deviceMacAddress?.let { return it.toString() }
        }
        // Older: a ScanResult or BluetoothDevice is returned.
        val scan = if (Build.VERSION.SDK_INT >= 33)
            data.getParcelableExtra(CompanionDeviceManager.EXTRA_DEVICE, ScanResult::class.java)
        else @Suppress("DEPRECATION") data.getParcelableExtra(CompanionDeviceManager.EXTRA_DEVICE)
        (scan as? ScanResult)?.device?.address?.let { return it }
        val dev = if (Build.VERSION.SDK_INT >= 33)
            data.getParcelableExtra(CompanionDeviceManager.EXTRA_DEVICE, BluetoothDevice::class.java)
        else @Suppress("DEPRECATION") data.getParcelableExtra(CompanionDeviceManager.EXTRA_DEVICE)
        return (dev as? BluetoothDevice)?.address
    }
}
