// BleControlClient.kt
// BLE control plane (GATT central). Small JSON messages only: capabilities,
// status, the pairing handshake, the session request, and commands. Bulk data
// goes over Wi-Fi. UUIDs match the firmware (ble.rs).
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.ble

import android.annotation.SuppressLint
import android.bluetooth.*
import android.content.Context
import android.util.Log
import app.tucklet.protocol.*
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.withTimeout
import java.util.UUID

object TuckletGatt {
    val SERVICE: UUID = UUID.fromString("F0CC0001-0000-1000-8000-00805F9B34FB")
    val STATUS: UUID = UUID.fromString("F0CC0002-0000-1000-8000-00805F9B34FB")
    val AUTH: UUID = UUID.fromString("F0CC0003-0000-1000-8000-00805F9B34FB")
    val SESSION: UUID = UUID.fromString("F0CC0004-0000-1000-8000-00805F9B34FB")
    val COMMAND: UUID = UUID.fromString("F0CC0005-0000-1000-8000-00805F9B34FB")
    val CAPS: UUID = UUID.fromString("F0CC0006-0000-1000-8000-00805F9B34FB")
    val CCCD: UUID = UUID.fromString("00002902-0000-1000-8000-00805F9B34FB")
}

/**
 * Wraps a BluetoothGatt connection to one Tucklet. All public methods are
 * suspend functions that bridge the (callback-based) Android GATT API.
 *
 * CONFIRM on-device: characteristic write types (WRITE vs WRITE_NO_RESPONSE)
 * and the exact notification timing; Android 13+ uses the typed
 * writeCharacteristic(char, value, type) overload used here.
 */
@SuppressLint("MissingPermission")
class BleControlClient(private val context: Context) {

    val isConnected = MutableStateFlow(false)
    val lastStatus = MutableStateFlow<StatusReport?>(null)
    /** Most recent challenge nonce the device pushed (to be signed for a session). */
    val lastNonce = MutableStateFlow<ByteArray?>(null)

    private var gatt: BluetoothGatt? = null
    private var connectDeferred: CompletableDeferred<Unit>? = null
    private val reads = HashMap<UUID, CompletableDeferred<ByteArray>>()
    private val writes = HashMap<UUID, CompletableDeferred<Unit>>()
    private val notifies = HashMap<UUID, CompletableDeferred<ByteArray>>()

    private val callback = object : BluetoothGattCallback() {
        override fun onConnectionStateChange(g: BluetoothGatt, status: Int, newState: Int) {
            if (newState == BluetoothProfile.STATE_CONNECTED) {
                g.discoverServices()
            } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                isConnected.value = false
            }
        }

        override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
            // Subscribe to STATUS + SESSION notifications so we receive battery
            // updates and the per-connection session nonce.
            enableNotify(g, TuckletGatt.STATUS)
            enableNotify(g, TuckletGatt.SESSION)
            isConnected.value = true
            connectDeferred?.complete(Unit)
        }

        override fun onCharacteristicRead(g: BluetoothGatt, ch: BluetoothGattCharacteristic, value: ByteArray, status: Int) {
            reads.remove(ch.uuid)?.complete(value)
        }

        override fun onCharacteristicWrite(g: BluetoothGatt, ch: BluetoothGattCharacteristic, status: Int) {
            writes.remove(ch.uuid)?.complete(Unit)
        }

        override fun onCharacteristicChanged(g: BluetoothGatt, ch: BluetoothGattCharacteristic, value: ByteArray) {
            when (ch.uuid) {
                TuckletGatt.STATUS ->
                    runCatching { TuckletJson.decodeFromString<StatusReport>(String(value)) }
                        .onSuccess { lastStatus.value = it }
                TuckletGatt.SESSION -> {
                    // {"nonce":"<hex>"} pushed on connect; or a SessionGrant in
                    // response to our write (handled by the notify deferred).
                    val text = String(value)
                    val nonceHex = Regex("\"nonce\"\\s*:\\s*\"([0-9a-fA-F]+)\"").find(text)?.groupValues?.get(1)
                    if (nonceHex != null) lastNonce.value = nonceHex.hexToBytes()
                    else notifies.remove(ch.uuid)?.complete(value)
                }
                TuckletGatt.AUTH -> notifies.remove(ch.uuid)?.complete(value)
                else -> {}
            }
        }
    }

    suspend fun connect(device: BluetoothDevice, timeoutMs: Long = 10_000) {
        val d = CompletableDeferred<Unit>()
        connectDeferred = d
        gatt = device.connectGatt(context, false, callback, BluetoothDevice.TRANSPORT_LE)
        withTimeout(timeoutMs) { d.await() }
    }

    fun disconnect() {
        gatt?.disconnect(); gatt?.close(); gatt = null; isConnected.value = false
    }

    suspend fun readCapabilities(): DeviceCapabilities =
        TuckletJson.decodeFromString(String(read(TuckletGatt.CAPS)))

    suspend fun readStatus(): StatusReport =
        TuckletJson.decodeFromString<StatusReport>(String(read(TuckletGatt.STATUS))).also { lastStatus.value = it }

    suspend fun pair(req: PairRequest): PairResponse {
        val resp = writeThenAwaitNotify(TuckletGatt.AUTH, TuckletJson.encodeToString(PairRequest.serializer(), req).toByteArray())
        return TuckletJson.decodeFromString(String(resp))
    }

    /** Write the session request and await the SessionGrant notification. */
    suspend fun openSession(req: SessionRequest): SessionGrant {
        val resp = writeThenAwaitNotify(TuckletGatt.SESSION, TuckletJson.encodeToString(SessionRequest.serializer(), req).toByteArray())
        return TuckletJson.decodeFromString(String(resp))
    }

    suspend fun sendCommand(cmd: Command) {
        write(TuckletGatt.COMMAND, TuckletJson.encodeToString(Command.serializer(), cmd).toByteArray())
    }

    // --- low-level helpers ---

    private suspend fun read(uuid: UUID, timeoutMs: Long = 5_000): ByteArray {
        val g = gatt ?: error("not connected")
        val ch = g.getService(TuckletGatt.SERVICE)?.getCharacteristic(uuid) ?: error("no characteristic $uuid")
        val d = CompletableDeferred<ByteArray>(); reads[uuid] = d
        g.readCharacteristic(ch)
        return withTimeout(timeoutMs) { d.await() }
    }

    private suspend fun write(uuid: UUID, bytes: ByteArray, timeoutMs: Long = 5_000) {
        val g = gatt ?: error("not connected")
        val ch = g.getService(TuckletGatt.SERVICE)?.getCharacteristic(uuid) ?: error("no characteristic $uuid")
        val d = CompletableDeferred<Unit>(); writes[uuid] = d
        g.writeCharacteristic(ch, bytes, BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT)
        withTimeout(timeoutMs) { d.await() }
    }

    private suspend fun writeThenAwaitNotify(uuid: UUID, bytes: ByteArray, timeoutMs: Long = 8_000): ByteArray {
        val n = CompletableDeferred<ByteArray>(); notifies[uuid] = n
        write(uuid, bytes)
        return withTimeout(timeoutMs) { n.await() }
    }

    private fun enableNotify(g: BluetoothGatt, uuid: UUID) {
        val ch = g.getService(TuckletGatt.SERVICE)?.getCharacteristic(uuid) ?: return
        g.setCharacteristicNotification(ch, true)
        val cccd = ch.getDescriptor(TuckletGatt.CCCD) ?: return
        g.writeDescriptor(cccd, BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
    }

    companion object { private const val TAG = "TuckletBLE" }
}

/** hex string -> bytes (even length). */
fun String.hexToBytes(): ByteArray {
    val clean = trim()
    require(clean.length % 2 == 0) { "odd hex length" }
    return ByteArray(clean.length / 2) { i ->
        ((Character.digit(clean[i * 2], 16) shl 4) + Character.digit(clean[i * 2 + 1], 16)).toByte()
    }
}
