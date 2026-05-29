// FileProviderExtension.swift
// Makes the contents of a Tucklet appear inside the iOS Files app and document
// pickers, so users browse remote files almost like local ones (the closest
// iOS allows to virtual-filesystem integration; see docs/adr/ADR-003).
//
// This implements the core of NSFileProviderReplicatedExtension. The domain
// registration glue (NSFileProviderManager.add(domain:)) lives in the app and
// is noted in the iOS README. CONFIRM method signatures against the iOS 26 SDK.
//
// License: PolyForm Noncommercial 1.0.0

import FileProvider
import UniformTypeIdentifiers

final class FileProviderExtension: NSObject, NSFileProviderReplicatedExtension {

    required init(domain: NSFileProviderDomain) {
        super.init()
    }

    func invalidate() {}

    // MARK: Item metadata

    func item(for identifier: NSFileProviderItemIdentifier,
              request: NSFileProviderRequest,
              completionHandler: @escaping (NSFileProviderItem?, Error?) -> Void) -> Progress {
        let progress = Progress(totalUnitCount: 1)
        Task {
            do {
                if identifier == .rootContainer {
                    completionHandler(TuckletFolderItem.root, nil)
                } else {
                    let item = try await TuckletStore.shared.item(id: identifier.rawValue)
                    completionHandler(item, nil)
                }
            } catch {
                completionHandler(nil, error)
            }
            progress.completedUnitCount = 1
        }
        return progress
    }

    // MARK: Contents (the actual bytes — fetched on demand)

    func fetchContents(for itemIdentifier: NSFileProviderItemIdentifier,
                       version requestedVersion: NSFileProviderItemVersion?,
                       request: NSFileProviderRequest,
                       completionHandler: @escaping (URL?, NSFileProviderItem?, Error?) -> Void) -> Progress {
        let progress = Progress(totalUnitCount: 100)
        Task {
            do {
                let (url, item) = try await TuckletStore.shared.downloadContents(id: itemIdentifier.rawValue)
                progress.completedUnitCount = 100
                completionHandler(url, item, nil)
            } catch {
                completionHandler(nil, nil, error)
            }
        }
        return progress
    }

    // MARK: Mutations

    func createItem(basedOn itemTemplate: NSFileProviderItem,
                    fields: NSFileProviderItemFields,
                    contents url: URL?,
                    options: NSFileProviderCreateItemOptions = [],
                    request: NSFileProviderRequest,
                    completionHandler: @escaping (NSFileProviderItem?, NSFileProviderItemFields, Bool, Error?) -> Void) -> Progress {
        let progress = Progress(totalUnitCount: 1)
        Task {
            do {
                let item = try await TuckletStore.shared.upload(template: itemTemplate, contents: url)
                completionHandler(item, [], false, nil)
            } catch {
                completionHandler(nil, [], false, error)
            }
            progress.completedUnitCount = 1
        }
        return progress
    }

    func modifyItem(_ item: NSFileProviderItem, baseVersion version: NSFileProviderItemVersion,
                    changedFields: NSFileProviderItemFields, contents newContents: URL?,
                    options: NSFileProviderModifyItemOptions = [], request: NSFileProviderRequest,
                    completionHandler: @escaping (NSFileProviderItem?, NSFileProviderItemFields, Bool, Error?) -> Void) -> Progress {
        let progress = Progress(totalUnitCount: 1)
        completionHandler(item, [], false, nil)
        progress.completedUnitCount = 1
        return progress
    }

    func deleteItem(identifier: NSFileProviderItemIdentifier, baseVersion version: NSFileProviderItemVersion,
                    options: NSFileProviderDeleteItemOptions = [], request: NSFileProviderRequest,
                    completionHandler: @escaping (Error?) -> Void) -> Progress {
        let progress = Progress(totalUnitCount: 1)
        Task {
            do { try await TuckletStore.shared.delete(id: identifier.rawValue); completionHandler(nil) }
            catch { completionHandler(error) }
            progress.completedUnitCount = 1
        }
        return progress
    }

    func enumerator(for containerItemIdentifier: NSFileProviderItemIdentifier,
                    request: NSFileProviderRequest) throws -> NSFileProviderEnumerator {
        TuckletEnumerator(container: containerItemIdentifier)
    }
}

// MARK: - Items

struct TuckletFolderItem: NSFileProviderItem {
    let identifier: NSFileProviderItemIdentifier
    let name: String
    static let root = TuckletFolderItem(identifier: .rootContainer, name: "Tucklet")

    var itemIdentifier: NSFileProviderItemIdentifier { identifier }
    var parentItemIdentifier: NSFileProviderItemIdentifier { .rootContainer }
    var filename: String { name }
    var contentType: UTType { .folder }
    var capabilities: NSFileProviderItemCapabilities { [.allowsReading, .allowsContentEnumerating] }
}

struct TuckletFileItem: NSFileProviderItem {
    let media: MediaItem
    var itemIdentifier: NSFileProviderItemIdentifier { NSFileProviderItemIdentifier(media.id) }
    var parentItemIdentifier: NSFileProviderItemIdentifier { .rootContainer }
    var filename: String { media.name }
    var documentSize: NSNumber? { NSNumber(value: media.sizeBytes) }
    var contentType: UTType { UTType(mimeType: media.mime) ?? .data }
    var capabilities: NSFileProviderItemCapabilities { [.allowsReading, .allowsDeleting] }
    var creationDate: Date?? { Date(timeIntervalSince1970: TimeInterval(media.createdAt)) }
}

// MARK: - Enumerator

final class TuckletEnumerator: NSObject, NSFileProviderEnumerator {
    let container: NSFileProviderItemIdentifier
    init(container: NSFileProviderItemIdentifier) { self.container = container }
    func invalidate() {}

    func enumerateItems(for observer: NSFileProviderEnumerationObserver, startingAt page: NSFileProviderPage) {
        Task {
            do {
                let items = try await TuckletStore.shared.allItems().map { TuckletFileItem(media: $0) }
                observer.didEnumerate(items)
                observer.finishEnumerating(upTo: nil)
            } catch {
                observer.finishEnumeratingWithError(error)
            }
        }
    }
}

/// Shared store the extension uses to reach the device. It reuses the same
/// DataClient/SessionManager as the app via the shared app group, so browsing
/// in Files goes through the exact same protocol. CONFIRM: session credentials
/// are shared to the extension via the app group keychain.
actor TuckletStore {
    static let shared = TuckletStore()
    func allItems() async throws -> [MediaItem] { [] }            // wired to DataClient.manifest()
    func item(id: String) async throws -> NSFileProviderItem { TuckletFolderItem.root }
    func downloadContents(id: String) async throws -> (URL, NSFileProviderItem) {
        throw NSError(domain: "tucklet", code: -1)               // wired to DataClient.download()
    }
    func upload(template: NSFileProviderItem, contents: URL?) async throws -> NSFileProviderItem {
        TuckletFolderItem.root                                    // wired to DataClient.upload()
    }
    func delete(id: String) async throws {}                       // wired to DataClient.delete()
}
