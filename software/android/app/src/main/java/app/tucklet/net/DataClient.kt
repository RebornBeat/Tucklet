// DataClient.kt
// The /v1 data API over HTTP, pinned to the charm's Wi-Fi Network and
// authenticated with the per-session bearer token. Mirrors httpd.rs.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.net

import android.net.Network
import android.util.Base64
import app.tucklet.protocol.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.asRequestBody
import java.io.File

class DataClient(
    network: Network,
    private val ip: String,
    private val token: String,
) {
    private val base = "http://$ip/v1"
    private val http: OkHttpClient = OkHttpClient.Builder()
        .socketFactory(network.socketFactory) // pin to the charm
        .build()

    private fun req(path: String): Request.Builder =
        Request.Builder().url("$base/$path").header("X-Tucklet-Token", token)

    suspend fun manifest(): Manifest = withContext(Dispatchers.IO) {
        http.newCall(req("manifest").get().build()).execute().use { r ->
            check(r.isSuccessful) { "manifest ${r.code}" }
            TuckletJson.decodeFromString(r.body!!.string())
        }
    }

    suspend fun thumbnail(id: String): ByteArray? = withContext(Dispatchers.IO) {
        http.newCall(req("thumb/$id").get().build()).execute().use { r ->
            if (!r.isSuccessful) null else r.body!!.bytes()
        }
    }

    suspend fun download(id: String, dest: File) = withContext(Dispatchers.IO) {
        http.newCall(req("file/$id").get().build()).execute().use { r ->
            check(r.isSuccessful) { "download ${r.code}" }
            dest.outputStream().use { out -> r.body!!.byteStream().copyTo(out) }
        }
    }

    /** Upload a local file; the firmware stores the full MediaItem (base64 in
     *  the X-Tucklet-Origin header) so it can rebuild the manifest + restore. */
    suspend fun upload(file: File, item: MediaItem) = withContext(Dispatchers.IO) {
        val itemJson = TuckletJson.encodeToString(MediaItem.serializer(), item).toByteArray()
        val originB64 = Base64.encodeToString(itemJson, Base64.NO_WRAP)
        val body = file.asRequestBody("application/octet-stream".toMediaType())
        val request = req("file").post(body).header("X-Tucklet-Origin", originB64).build()
        http.newCall(request).execute().use { r -> check(r.isSuccessful) { "upload ${r.code}" } }
    }

    suspend fun delete(id: String) = withContext(Dispatchers.IO) {
        http.newCall(req("file/$id").delete().build()).execute().use { r ->
            check(r.isSuccessful) { "delete ${r.code}" }
        }
    }

    suspend fun restoreOrigin(id: String): OriginMetadata = withContext(Dispatchers.IO) {
        http.newCall(req("restore/$id").post(ByteArray(0).toRequestBody()).build()).execute().use { r ->
            check(r.isSuccessful) { "restore ${r.code}" }
            TuckletJson.decodeFromString(r.body!!.string())
        }
    }
}

private fun ByteArray.toRequestBody(): RequestBody =
    RequestBody.create(null, this)
