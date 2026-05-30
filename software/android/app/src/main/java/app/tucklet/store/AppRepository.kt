// AppRepository.kt
// The shared orchestrator used by BOTH the foreground UI (AppViewModel) and the
// background TrickleWorker. Owns the BLE client, Wi-Fi connector, session,
// photo source, and pairing; exposes state as flows and the high-level actions.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.store

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.content.Context
import android.graphics.Bitmap
import app.tucklet.ble.BleControlClient
import app.tucklet.ble.hexToBytes
import app.tucklet.core.LinkProfile
import app.tucklet.core.Trickle
import app.tucklet.net.DataClient
import app.tucklet.net.SessionManager
import app.tucklet.pairing.PairingManager
import app.tucklet.photos.PhotoSource
import app.tucklet.protocol.*
import app.tucklet.wifi.WifiConnector
import kotlinx.coroutines.flow.MutableStateFlow
import java.io.File

@SuppressLint("MissingPermission")
class AppRepository(private val appContext: Context) {

    enum class Conn { IDLE, CONNECTING, CONNECTED, ERROR }

    val connection = MutableStateFlow(Conn.IDLE)
    val errorText = MutableStateFlow<String?>(null)
    val capabilities = MutableStateFlow<DeviceCapabilities?>(null)
    val status = MutableStateFlow<StatusReport?>(null)
    val manifest = MutableStateFlow<Manifest?>(null)
    val onPhoneItems = MutableStateFlow<List<MediaItem>>(emptyList())
    val isPaired = MutableStateFlow(false)
    val lastOffloadIds = MutableStateFlow<List<String>>(emptyList())

    val ble = BleControlClient(appContext)
    private val wifi = WifiConnector(appContext)
    private val sessions = SessionManager(ble, wifi)
    val photos = PhotoSource(appContext)
    val pairing = PairingManager(appContext)
    private val crypto = Crypto(appContext)
    private var data: DataClient? = null

    private val adapter: BluetoothAdapter? get() =
        (appContext.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager)?.adapter

    init { isPaired.value = pairing.isAssociated }

    // --- connection -------------------------------------------------------

    /** Connect to a specific device (from the CompanionDeviceManager chooser),
     *  enroll cryptographically, then load. */
    suspend fun enrollAndConnect(macAddress: String) {
        connection.value = Conn.CONNECTING
        try {
            val device = adapter?.getRemoteDevice(macAddress) ?: error("no bluetooth adapter")
            ble.connect(device)
            val pub = crypto.devicePublicKeyHex()
            val resp = ble.pair(PairRequest(pub, android.os.Build.MODEL ?: "Android"))
            if (resp.paired) {
                isPaired.value = true
                finishConnect()
            } else {
                connection.value = Conn.ERROR; errorText.value = resp.reason ?: "pairing not confirmed"
            }
        } catch (t: Throwable) { fail(t) }
    }

    /** Silent reconnect to an already-associated charm. */
    suspend fun connect() {
        val mac = pairing.associations.firstOrNull()
        if (mac == null) { isPaired.value = false; return }
        connection.value = Conn.CONNECTING
        try {
            val device = adapter?.getRemoteDevice(mac) ?: error("no bluetooth adapter")
            ble.connect(device)
            finishConnect()
        } catch (t: Throwable) { fail(t) }
    }

    private suspend fun finishConnect() {
        capabilities.value = ble.readCapabilities()
        status.value = ble.readStatus()
        isPaired.value = true
        connection.value = Conn.CONNECTED
        refreshPhoneLibrary()
    }

    private fun fail(t: Throwable) {
        connection.value = Conn.ERROR; errorText.value = t.message ?: t.toString()
    }

    /** Open the data session (sign the nonce) + load the on-Tucklet manifest. */
    suspend fun loadLibrary() {
        val caps = capabilities.value ?: return
        try {
            val sig = crypto.signSessionChallenge(ble.lastNonce.value)
            data = sessions.openDataSession(caps, sig)
            manifest.value = data?.manifest()
        } catch (t: Throwable) { fail(t) }
    }

    suspend fun refreshPhoneLibrary() {
        onPhoneItems.value = photos.cameraRollItems()
    }

    // --- unified library --------------------------------------------------

    fun libraryGroups(): List<Pair<String, List<MediaItem>>> {
        val all = onPhoneItems.value + (manifest.value?.items ?: emptyList())
        return all.groupBy { it.origin.app }.toSortedMap().map { (app, items) ->
            app to items.sortedByDescending { it.createdAt }
        }
    }

    fun pendingBackupCount(): Int {
        val onTucklet = (manifest.value?.items ?: emptyList()).map { it.id }.toSet()
        return onPhoneItems.value.count { it.id !in onTucklet }
    }

    suspend fun thumbnail(item: MediaItem): Bitmap? = when (item.itemState) {
        is ItemState.OnPhone -> photos.thumbnail(item.id)
        else -> data?.thumbnail(item.id)?.let { bytes ->
            android.graphics.BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
        }
    }

    // --- transfers --------------------------------------------------------

    fun linkProfile(): LinkProfile {
        val caps = capabilities.value ?: return LinkProfile.C5_WIRELESS
        val t = sessions.usableTransports(caps).firstOrNull() ?: DataTransport.SOFT_AP
        return LinkProfile.profileFor(caps.radio, t)
    }

    suspend fun offloadItem(item: MediaItem) {
        val d = data ?: error("no session")
        val file = photos.exportOriginal(item.id, item.name)
        try { d.upload(file, item) } finally { file.delete() }
    }

    suspend fun loadItem(item: MediaItem, policy: TemporaryPolicy) {
        val d = data ?: error("no session")
        val dest = File(appContext.cacheDir, item.name)
        d.download(item.id, dest)
        if (policy == TemporaryPolicy.KEEP) photos.restoreToPhone(dest, item)
    }

    suspend fun finishOffload(deletedIds: List<String>) {
        lastOffloadIds.value = deletedIds
        if (deletedIds.isNotEmpty()) { photos.deleteFromPhone(deletedIds); refreshPhoneLibrary() }
        manifest.value = data?.manifest()
    }

    suspend fun undoLastOffload() {
        val d = data ?: return
        for (id in lastOffloadIds.value) {
            val item = manifest.value?.items?.firstOrNull { it.id == id } ?: continue
            val dest = File(appContext.cacheDir, item.name)
            runCatching { d.download(id, dest); photos.restoreToPhone(dest, item) }
        }
        lastOffloadIds.value = emptyList()
        refreshPhoneLibrary()
    }

    suspend fun restore(item: MediaItem) {
        val d = data ?: error("no session")
        val origin = d.restoreOrigin(item.id)
        val dest = File(appContext.cacheDir, item.name)
        d.download(item.id, dest)
        // restoreToPhone uses item.origin; rebuild with the device-reported origin.
        photos.restoreToPhone(dest, item.copy(origin = origin))
        refreshPhoneLibrary()
    }

    suspend fun delete(item: MediaItem) {
        runCatching { data?.delete(item.id) }
        manifest.value = data?.manifest()
    }

    // --- trickle ----------------------------------------------------------

    fun trickleConditions(phoneCharging: Boolean, phoneIdle: Boolean): Trickle.Conditions =
        Trickle.Conditions(
            phoneInRange = connection.value == Conn.CONNECTED,
            phoneIdle = phoneIdle,
            charging = phoneCharging,
            batteryPercent = status.value?.batteryPercent ?: 0,
            pendingItems = pendingBackupCount(),
        )

    suspend fun trickleBackup(batchSize: Int): Int {
        if (data == null) loadLibrary()
        val onTucklet = (manifest.value?.items ?: emptyList()).map { it.id }.toSet()
        val pending = onPhoneItems.value.filter { it.id !in onTucklet }.take(batchSize)
        var done = 0
        for (item in pending) {
            runCatching { offloadItem(item) }.onSuccess { done++ }.onFailure { return done }
        }
        if (done > 0) manifest.value = data?.manifest()
        return done
    }

    fun endSession() { sessions.close(); data = null }

    suspend fun forget() {
        pairing.associations.forEach { pairing.disassociate(it) }
        endSession(); ble.disconnect()
        isPaired.value = false; connection.value = Conn.IDLE
        capabilities.value = null; status.value = null; manifest.value = null
    }
}

/**
 * Crypto seam — the one security primitive, now real and shared in standard
 * (RFC 8032 Ed25519, interoperable with the firmware's verify and the iOS/desktop
 * clients). A software Ed25519 key (BouncyCastle) is generated on first run and
 * sealed in EncryptedSharedPreferences; its public key is enrolled at pairing and
 * each session nonce is signed with the private key.
 *
 * CONFIRM: Android Keystore Ed25519 support is uneven across versions, so the
 * sealed-software-key approach is the portable choice; swap to Keystore/StrongBox
 * where available.
 */
class Crypto(context: Context) {
    private val prefs = run {
        val master = androidx.security.crypto.MasterKey.Builder(context)
            .setKeyScheme(androidx.security.crypto.MasterKey.KeyScheme.AES256_GCM)
            .build()
        androidx.security.crypto.EncryptedSharedPreferences.create(
            context,
            "tucklet_identity",
            master,
            androidx.security.crypto.EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            androidx.security.crypto.EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    private fun loadOrCreatePrivate(): org.bouncycastle.crypto.params.Ed25519PrivateKeyParameters {
        prefs.getString(KEY_SEED, null)?.let { hex ->
            return org.bouncycastle.crypto.params.Ed25519PrivateKeyParameters(hex.hexToBytes(), 0)
        }
        val priv = org.bouncycastle.crypto.params.Ed25519PrivateKeyParameters(java.security.SecureRandom())
        prefs.edit().putString(KEY_SEED, priv.encoded.toHex()).apply()
        return priv
    }

    /** Hex public key (32 bytes -> 64 hex) to enroll with the device. */
    fun devicePublicKeyHex(): String = loadOrCreatePrivate().generatePublicKey().encoded.toHex()

    /** Sign the RAW nonce bytes (already hex-decoded from the SESSION notify). */
    fun signSessionChallenge(nonce: ByteArray?): String {
        require(nonce != null && nonce.isNotEmpty()) { "no nonce" }
        val signer = org.bouncycastle.crypto.signers.Ed25519Signer()
        signer.init(true, loadOrCreatePrivate())
        signer.update(nonce, 0, nonce.size)
        return signer.generateSignature().toHex()
    }

    companion object { private const val KEY_SEED = "ed25519_seed_hex" }
}

private fun ByteArray.toHex(): String = joinToString("") { "%02x".format(it) }
