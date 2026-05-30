// PhotoKitSource.swift
// The phone-side photo library bridge. Turns the camera roll into MediaItems
// (with round-trip OriginMetadata), supplies thumbnails, exports originals for
// upload, and re-imports files on restore — the "On phone" half of the Library.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation
import Photos
import UIKit

/// Read/write access to the device photo library, expressed in the app's own
/// MediaItem vocabulary so the UI treats on-phone and on-Tucklet items the same.
@MainActor
public final class PhotoKitSource {
    private let imageManager = PHCachingImageManager()
    private let deviceName: String

    public init(deviceName: String) {
        self.deviceName = deviceName
    }

    /// Ask for read/write access. Returns true if usable.
    public func requestAccess() async -> Bool {
        let status = await PHPhotoLibrary.requestAuthorization(for: .readWrite)
        return status == .authorized || status == .limited
    }

    /// Enumerate camera-roll photos & videos as on-phone MediaItems, newest
    /// first. The asset's localIdentifier becomes the MediaItem id so we can
    /// resolve back for export/restore.
    public func cameraRollItems(limit: Int = 500) async -> [MediaItem] {
        let opts = PHFetchOptions()
        opts.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]
        opts.fetchLimit = limit
        let assets = PHAsset.fetchAssets(with: opts)

        // Build a localIdentifier -> album-name map for nicer origin grouping.
        let albumOf = albumIndex()

        var out: [MediaItem] = []
        assets.enumerateObjects { asset, _, _ in
            let id = asset.localIdentifier
            let isVideo = asset.mediaType == .video
            let name = (asset.value(forKey: "filename") as? String)
                ?? (PHAssetResource.assetResources(for: asset).first?.originalFilename)
                ?? "\(isVideo ? "VID" : "IMG")_\(id.prefix(8)).\(isVideo ? "mov" : "heic")"
            let album = albumOf[id]
            let origin = OriginMetadata(
                platform: .ios,
                app: album ?? "Camera",
                collection: album.map { "Albums/\($0)" } ?? "DCIM/Camera",
                album: album,
                deviceName: self.deviceName
            )
            let item = MediaItem(
                id: id,
                name: name,
                sizeBytes: UInt64(self.estimateBytes(asset)),
                mime: isVideo ? "video/quicktime" : "image/heic",
                createdAt: Int64(asset.creationDate?.timeIntervalSince1970 ?? 0),
                origin: origin,
                state: .onPhone,
                checksum: nil
            )
            out.append(item)
        }
        return out
    }

    /// Thumbnail for an on-phone asset id.
    public func thumbnail(for assetId: String, size: CGSize = CGSize(width: 200, height: 200)) async -> UIImage? {
        guard let asset = PHAsset.fetchAssets(withLocalIdentifiers: [assetId], options: nil).firstObject else {
            return nil
        }
        return await withCheckedContinuation { cont in
            let opts = PHImageRequestOptions()
            opts.deliveryMode = .opportunistic
            opts.isNetworkAccessAllowed = true
            imageManager.requestImage(for: asset, targetSize: size, contentMode: .aspectFill, options: opts) { image, _ in
                cont.resume(returning: image)
            }
        }
    }

    /// Export an on-phone asset's original bytes to a temp file URL for upload.
    public func exportOriginal(assetId: String) async throws -> URL {
        guard let asset = PHAsset.fetchAssets(withLocalIdentifiers: [assetId], options: nil).firstObject else {
            throw PhotoError.notFound
        }
        let resources = PHAssetResource.assetResources(for: asset)
        guard let resource = resources.first(where: { $0.type == .photo || $0.type == .video || $0.type == .fullSizePhoto || $0.type == .fullSizeVideo }) ?? resources.first else {
            throw PhotoError.noResource
        }
        let dest = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
            .appendingPathExtension((resource.originalFilename as NSString).pathExtension)
        if FileManager.default.fileExists(atPath: dest.path) {
            try FileManager.default.removeItem(at: dest)
        }
        let options = PHAssetResourceRequestOptions()
        options.isNetworkAccessAllowed = true
        try await PHAssetResourceManager.default().writeData(for: resource, toFile: dest, options: options)
        return dest
    }

    /// Delete on-phone originals after a confirmed offload (the "free up space"
    /// completion). Must be wrapped in a PHPhotoLibrary change request and will
    /// prompt the user for deletion confirmation — that prompt is an iOS
    /// guarantee we surface honestly rather than try to suppress.
    public func deleteFromPhone(assetIds: [String]) async throws {
        let assets = PHAsset.fetchAssets(withLocalIdentifiers: assetIds, options: nil)
        try await PHPhotoLibrary.shared().performChanges {
            PHAssetChangeRequest.deleteAssets(assets)
        }
    }

    /// Restore a downloaded file back into the photo library, placing it into
    /// the album named in its origin metadata when possible (round-trip).
    public func restoreToPhone(fileURL: URL, origin: OriginMetadata, isVideo: Bool) async throws {
        try await PHPhotoLibrary.shared().performChanges {
            let req = PHAssetCreationRequest.forAsset()
            let type: PHAssetResourceType = isVideo ? .video : .photo
            req.addResource(with: type, fileURL: fileURL, options: nil)
            // Add to the origin album if it exists (best-effort).
            if let albumName = origin.album,
               let collection = Self.findAlbum(named: albumName),
               let placeholder = req.placeholderForCreatedAsset,
               let addReq = PHAssetCollectionChangeRequest(for: collection) {
                addReq.addAssets([placeholder] as NSArray)
            }
        }
    }

    // MARK: - helpers

    private func estimateBytes(_ asset: PHAsset) -> Int {
        // PHAsset doesn't expose exact bytes cheaply; estimate from pixels /
        // duration so the transfer-time estimate is reasonable before export.
        if asset.mediaType == .video {
            return Int(asset.duration * 5_000_000) // ~5 MB/s of HEVC video
        }
        let px = asset.pixelWidth * asset.pixelHeight
        return max(px / 4, 1_500_000) // HEIC ~ pixels/4 bytes, min ~1.5MB
    }

    private func albumIndex() -> [String: String] {
        var map: [String: String] = [:]
        let albums = PHAssetCollection.fetchAssetCollections(with: .album, subtype: .any, options: nil)
        albums.enumerateObjects { collection, _, _ in
            let title = collection.localizedTitle ?? "Album"
            let assets = PHAsset.fetchAssets(in: collection, options: nil)
            assets.enumerateObjects { asset, _, _ in
                map[asset.localIdentifier] = title
            }
        }
        return map
    }

    private static func findAlbum(named: String) -> PHAssetCollection? {
        let opts = PHFetchOptions()
        opts.predicate = NSPredicate(format: "localizedTitle = %@", named)
        return PHAssetCollection.fetchAssetCollections(with: .album, subtype: .any, options: opts).firstObject
    }

    public enum PhotoError: Error { case notFound, noResource }
}
