// DataClient.swift
// Data plane over the device's local HTTP API (carried by SoftAP or Wi-Fi
// Aware). Browses metadata/thumbnails only; full bytes move on explicit action.
//
// License: PolyForm Noncommercial 1.0.0

import Foundation

public struct DataClient: Sendable {
    let baseURL: URL
    let token: String
    private let session: URLSession

    public init(baseURL: URL, token: String) {
        self.baseURL = baseURL
        self.token = token
        let cfg = URLSessionConfiguration.ephemeral
        cfg.waitsForConnectivity = true
        cfg.timeoutIntervalForRequest = 30
        self.session = URLSession(configuration: cfg)
    }

    private func request(_ path: String, method: String = "GET") -> URLRequest {
        var r = URLRequest(url: baseURL.appendingPathComponent(path))
        r.httpMethod = method
        r.setValue(token, forHTTPHeaderField: "X-Tucklet-Token")
        return r
    }

    /// GET /v1/manifest — metadata + free/total space.
    public func manifest() async throws -> Manifest {
        let (data, resp) = try await session.data(for: request("v1/manifest"))
        try Self.check(resp)
        return try JSONDecoder().decode(Manifest.self, from: data)
    }

    /// GET /v1/thumb/{id} — small preview for the gallery.
    public func thumbnail(id: String) async throws -> Data {
        let (data, resp) = try await session.data(for: request("v1/thumb/\(id)"))
        try Self.check(resp)
        return data
    }

    /// GET /v1/file/{id} — full file streamed to a local URL (supports resume).
    public func download(id: String, to destination: URL) async throws {
        let (tmp, resp) = try await session.download(for: request("v1/file/\(id)"))
        try Self.check(resp)
        if FileManager.default.fileExists(atPath: destination.path) {
            try FileManager.default.removeItem(at: destination)
        }
        try FileManager.default.moveItem(at: tmp, to: destination)
    }

    /// POST /v1/file — upload a local file with its origin metadata.
    /// POST /v1/file — upload a local file. The firmware stores the full
    /// MediaItem (origin, name, mime, created_at, id) so it can rebuild the
    /// manifest and support round-trip restore; we send it base64 in the header
    /// and the body is pure bytes (streamable, large-file safe).
    public func upload(fileURL: URL, origin: OriginMetadata, item: MediaItem) async throws {
        var r = request("v1/file", method: "POST")
        r.setValue("application/octet-stream", forHTTPHeaderField: "Content-Type")
        // The firmware stores this MediaItem verbatim (and forces its state to
        // onTucklet). `origin` is already part of `item`; passed separately for
        // call-site clarity.
        _ = origin
        let itemJSON = try JSONEncoder().encode(item)
        r.setValue(itemJSON.base64EncodedString(), forHTTPHeaderField: "X-Tucklet-Origin")
        let (_, resp) = try await session.upload(for: r, fromFile: fileURL)
        try Self.check(resp)
    }

    /// DELETE /v1/file/{id}.
    public func delete(id: String) async throws {
        let (_, resp) = try await session.data(for: request("v1/file/\(id)", method: "DELETE"))
        try Self.check(resp)
    }

    /// POST /v1/restore/{id} — get origin metadata so the app can put the file
    /// back into the exact album/app it came from.
    public func restoreOrigin(id: String) async throws -> OriginMetadata {
        let (data, resp) = try await session.data(for: request("v1/restore/\(id)", method: "POST"))
        try Self.check(resp)
        return try JSONDecoder().decode(OriginMetadata.self, from: data)
    }

    private static func check(_ resp: URLResponse) throws {
        guard let http = resp as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
            throw URLError(.badServerResponse)
        }
    }
}
