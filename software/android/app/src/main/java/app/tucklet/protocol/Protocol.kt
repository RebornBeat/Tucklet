// Protocol.kt
// Kotlin mirror of `tucklet-proto`. Wire-compatible with the firmware: snake_case
// field names and the same shapes (incl. the flattened ItemState).
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.protocol

import kotlinx.serialization.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*

const val PROTOCOL_VERSION: Int = 1
typealias EpochSeconds = Long

@Serializable
enum class RadioKind {
    @SerialName("single_c5") SINGLE_C5,
    @SerialName("dual_c5") DUAL_C5,
}

@Serializable
enum class DataTransport {
    @SerialName("soft_ap") SOFT_AP,
    @SerialName("wifi_aware") WIFI_AWARE,
    @SerialName("wired_usb_hs") WIRED_USB_HS,
}

@Serializable
enum class Platform {
    @SerialName("ios") IOS,
    @SerialName("android") ANDROID,
    @SerialName("desktop") DESKTOP,
}

/** StorageKind matches serde's default enum encoding: the unit variant is the
 *  string "micro_sd"; the struct variant is {"emmc":{"capacity_gib":N}}. */
@Serializable(with = StorageKindSerializer::class)
sealed class StorageKind {
    object MicroSd : StorageKind()
    data class Emmc(val capacityGib: Int) : StorageKind()

    val label: String
        get() = when (this) {
            is MicroSd -> "microSD"
            is Emmc -> "$capacityGib GB built-in"
        }
}

object StorageKindSerializer : KSerializer<StorageKind> {
    override val descriptor: SerialDescriptor =
        buildClassSerialDescriptor("StorageKind")

    override fun serialize(encoder: Encoder, value: StorageKind) {
        val json = (encoder as JsonEncoder)
        when (value) {
            is StorageKind.MicroSd -> json.encodeJsonElement(JsonPrimitive("micro_sd"))
            is StorageKind.Emmc -> json.encodeJsonElement(
                buildJsonObject {
                    putJsonObject("emmc") { put("capacity_gib", value.capacityGib) }
                }
            )
        }
    }

    override fun deserialize(decoder: Decoder): StorageKind {
        val el = (decoder as JsonDecoder).decodeJsonElement()
        return when (el) {
            is JsonPrimitive -> if (el.content == "micro_sd") StorageKind.MicroSd
                else error("unknown storage ${el.content}")
            is JsonObject -> {
                val cap = el["emmc"]?.jsonObject?.get("capacity_gib")?.jsonPrimitive?.int
                    ?: error("emmc missing capacity_gib")
                StorageKind.Emmc(cap)
            }
            else -> error("unexpected storage encoding")
        }
    }
}

@Serializable
data class DeviceCapabilities(
    @SerialName("protocol_version") val protocolVersion: Int,
    @SerialName("firmware_version") val firmwareVersion: String,
    val radio: RadioKind,
    val storage: StorageKind,
    val transports: List<DataTransport>,
    @SerialName("device_id") val deviceId: String,
) {
    fun supports(t: DataTransport) = transports.contains(t)
    fun bestWireless(): DataTransport? =
        transports.firstOrNull { it == DataTransport.WIFI_AWARE }
            ?: transports.firstOrNull { it == DataTransport.SOFT_AP }
}

/** Where an item lives, in the exact vocabulary the UI shows. */
sealed class ItemState {
    object OnPhone : ItemState()
    object OnTucklet : ItemState()
    data class Temporary(val expiresAt: EpochSeconds?) : ItemState()

    val label: String
        get() = when (this) {
            is OnPhone -> "On phone"
            is OnTucklet -> "On Tucklet"
            is Temporary -> "Temporary"
        }
}

@Serializable
enum class TemporaryPolicy(val seconds: Long?) {
    @SerialName("one_hour") ONE_HOUR(3_600),
    @SerialName("one_day") ONE_DAY(86_400),
    @SerialName("one_week") ONE_WEEK(604_800),
    @SerialName("keep") KEEP(null);

    val label: String
        get() = when (this) {
            ONE_HOUR -> "1 hour"; ONE_DAY -> "1 day"; ONE_WEEK -> "1 week"; KEEP -> "Keep"
        }
}

@Serializable
data class OriginMetadata(
    val platform: Platform,
    val app: String,
    val collection: String,
    val album: String? = null,
    @SerialName("device_name") val deviceName: String,
)

/** MediaItem with the ItemState FLATTENED to the top level (state + expires_at),
 *  matching the firmware's #[serde(flatten)]. The convenient `itemState` is
 *  derived; the serialized fields are `state`/`expires_at`. */
@Serializable
data class MediaItem(
    val id: String,
    val name: String,
    @SerialName("size_bytes") val sizeBytes: Long,
    val mime: String,
    @SerialName("created_at") val createdAt: EpochSeconds,
    val origin: OriginMetadata,
    val state: String,                              // "on_phone"|"on_tucklet"|"temporary"
    @SerialName("expires_at") val expiresAt: EpochSeconds? = null,
    val checksum: String? = null,
) {
    val isImage get() = mime.startsWith("image/")
    val isVideo get() = mime.startsWith("video/")

    val itemState: ItemState
        get() = when (state) {
            "on_phone" -> ItemState.OnPhone
            "on_tucklet" -> ItemState.OnTucklet
            "temporary" -> ItemState.Temporary(expiresAt)
            else -> ItemState.OnPhone
        }

    companion object {
        fun onPhone(
            id: String, name: String, sizeBytes: Long, mime: String,
            createdAt: EpochSeconds, origin: OriginMetadata,
        ) = MediaItem(id, name, sizeBytes, mime, createdAt, origin, "on_phone", null, null)
    }
}

@Serializable
data class Manifest(
    val items: List<MediaItem>,
    @SerialName("free_bytes") val freeBytes: Long,
    @SerialName("total_bytes") val totalBytes: Long,
)

@Serializable
data class StatusReport(
    @SerialName("battery_percent") val batteryPercent: Int,
    val charging: Boolean,
    @SerialName("free_bytes") val freeBytes: Long,
    @SerialName("total_bytes") val totalBytes: Long,
    @SerialName("card_present") val cardPresent: Boolean,
    @SerialName("firmware_version") val firmwareVersion: String,
)

@Serializable
data class PairRequest(
    @SerialName("phone_pubkey") val phonePubkey: String,
    @SerialName("phone_name") val phoneName: String,
)

@Serializable
data class PairResponse(
    val paired: Boolean,
    @SerialName("device_pubkey") val devicePubkey: String? = null,
    val reason: String? = null,
)

@Serializable
data class SessionRequest(
    @SerialName("challenge_signature") val challengeSignature: String,
    val transport: DataTransport,
)

@Serializable
data class SessionGrant(
    @SerialName("ssid_or_service") val ssidOrService: String,
    val psk: String,
    val ip: String,
    val token: String,
    @SerialName("ttl_seconds") val ttlSeconds: Int,
)

@Serializable
enum class Command {
    @SerialName("sleep") SLEEP,
    @SerialName("factory_reset_confirm") FACTORY_RESET_CONFIRM,
    @SerialName("begin_trickle") BEGIN_TRICKLE,
}

@Serializable
enum class TransferKind { @SerialName("offload") OFFLOAD, @SerialName("load") LOAD }

@Serializable
enum class TransferMode { @SerialName("batch") BATCH, @SerialName("trickle") TRICKLE }

@Serializable
data class TransferItem(
    val id: String,
    @SerialName("size_bytes") val sizeBytes: Long,
    val mime: String,
)

@Serializable
data class TransferProgress(
    @SerialName("items_total") val itemsTotal: Int,
    @SerialName("items_done") val itemsDone: Int,
    @SerialName("bytes_total") val bytesTotal: Long,
    @SerialName("bytes_done") val bytesDone: Long,
    @SerialName("eta_seconds") val etaSeconds: Int,
    @SerialName("throughput_bps") val throughputBps: Long,
)

/** Shared JSON config (lenient like the device). */
val TuckletJson = Json {
    ignoreUnknownKeys = true
    encodeDefaults = false
    explicitNulls = false
}
