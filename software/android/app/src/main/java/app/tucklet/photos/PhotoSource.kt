// PhotoSource.kt
// The phone-side photo library bridge over MediaStore: enumerate the camera
// roll as MediaItems (with round-trip origin), supply thumbnails, export
// originals for upload, and re-import on restore.
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.photos

import android.content.ContentResolver
import android.content.ContentUris
import android.content.ContentValues
import android.content.Context
import android.graphics.Bitmap
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import android.util.Size
import app.tucklet.protocol.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

class PhotoSource(context: Context) {
    private val resolver: ContentResolver = context.contentResolver
    private val deviceName: String = Build.MODEL ?: "Android phone"
    private val cacheDir: File = context.cacheDir

    /** Enumerate images + videos newest-first as on-phone MediaItems. The
     *  MediaStore content URI string is the MediaItem id (resolvable later). */
    suspend fun cameraRollItems(limit: Int = 500): List<MediaItem> = withContext(Dispatchers.IO) {
        val out = ArrayList<MediaItem>(limit)
        out += query(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, isVideo = false, limit)
        out += query(MediaStore.Video.Media.EXTERNAL_CONTENT_URI, isVideo = true, limit)
        out.sortedByDescending { it.createdAt }.take(limit)
    }

    private fun query(collection: Uri, isVideo: Boolean, limit: Int): List<MediaItem> {
        val projection = arrayOf(
            MediaStore.MediaColumns._ID,
            MediaStore.MediaColumns.DISPLAY_NAME,
            MediaStore.MediaColumns.SIZE,
            MediaStore.MediaColumns.MIME_TYPE,
            MediaStore.MediaColumns.DATE_ADDED,
            MediaStore.MediaColumns.BUCKET_DISPLAY_NAME,
            MediaStore.MediaColumns.RELATIVE_PATH,
        )
        val sort = "${MediaStore.MediaColumns.DATE_ADDED} DESC"
        val items = ArrayList<MediaItem>()
        resolver.query(collection, projection, null, null, sort)?.use { c ->
            val idCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns._ID)
            val nameCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME)
            val sizeCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns.SIZE)
            val mimeCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns.MIME_TYPE)
            val dateCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns.DATE_ADDED)
            val bucketCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns.BUCKET_DISPLAY_NAME)
            val pathCol = c.getColumnIndexOrThrow(MediaStore.MediaColumns.RELATIVE_PATH)
            while (c.moveToNext() && items.size < limit) {
                val id = c.getLong(idCol)
                val uri = ContentUris.withAppendedId(collection, id)
                val bucket = c.getString(bucketCol) ?: if (isVideo) "Movies" else "Camera"
                val origin = OriginMetadata(
                    platform = Platform.ANDROID,
                    app = bucket,
                    collection = c.getString(pathCol) ?: (if (isVideo) "DCIM/Camera" else "DCIM/Camera"),
                    album = bucket,
                    deviceName = deviceName,
                )
                items += MediaItem.onPhone(
                    id = uri.toString(),
                    name = c.getString(nameCol) ?: "$id",
                    sizeBytes = c.getLong(sizeCol),
                    mime = c.getString(mimeCol) ?: if (isVideo) "video/mp4" else "image/jpeg",
                    createdAt = c.getLong(dateCol),
                    origin = origin,
                )
            }
        }
        return items
    }

    /** Thumbnail bitmap for an on-phone item (its content URI string). */
    suspend fun thumbnail(itemId: String, size: Int = 200): Bitmap? = withContext(Dispatchers.IO) {
        runCatching {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                resolver.loadThumbnail(Uri.parse(itemId), Size(size, size), null)
            } else null
        }.getOrNull()
    }

    /** Copy an on-phone original to a temp file for upload. */
    suspend fun exportOriginal(itemId: String, displayName: String): File = withContext(Dispatchers.IO) {
        val uri = Uri.parse(itemId)
        val dest = File(cacheDir, "${System.nanoTime()}_$displayName")
        resolver.openInputStream(uri)!!.use { input ->
            dest.outputStream().use { output -> input.copyTo(output) }
        }
        dest
    }

    /** Re-import a downloaded file into MediaStore (round-trip restore). */
    suspend fun restoreToPhone(file: File, item: MediaItem) = withContext(Dispatchers.IO) {
        val isVideo = item.isVideo
        val collection = if (isVideo) MediaStore.Video.Media.EXTERNAL_CONTENT_URI
            else MediaStore.Images.Media.EXTERNAL_CONTENT_URI
        val relPath = item.origin.collection.ifBlank { if (isVideo) "Movies/Tucklet" else "Pictures/Tucklet" }
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, item.name)
            put(MediaStore.MediaColumns.MIME_TYPE, item.mime)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(MediaStore.MediaColumns.RELATIVE_PATH, relPath)
                put(MediaStore.MediaColumns.IS_PENDING, 1)
            }
        }
        val uri = resolver.insert(collection, values) ?: error("insert failed")
        resolver.openOutputStream(uri)!!.use { out -> file.inputStream().use { it.copyTo(out) } }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            values.clear(); values.put(MediaStore.MediaColumns.IS_PENDING, 0)
            resolver.update(uri, values, null, null)
        }
    }

    /**
     * Delete on-phone originals after a confirmed offload. On Android 11+ this
     * may require a user-confirmed delete request (MediaStore.createDeleteRequest);
     * the caller surfaces that intent. Here we attempt the direct delete which
     * works for app-owned media and pre-30 devices.
     *
     * CONFIRM: wire createDeleteRequest(...) + IntentSenderForResult for items
     * the app doesn't own on Android 11+.
     */
    suspend fun deleteFromPhone(itemIds: List<String>): Int = withContext(Dispatchers.IO) {
        var n = 0
        for (id in itemIds) {
            n += runCatching { resolver.delete(Uri.parse(id), null, null) }.getOrDefault(0)
        }
        n
    }
}
