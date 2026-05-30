// api.ts — typed bridge to the Rust commands. Types mirror tucklet-proto; the
// item `state` is flattened (state + expires_at), matching the firmware wire.
// License: PolyForm Noncommercial 1.0.0
import { invoke } from "@tauri-apps/api/core";

export type Platform = "ios" | "android" | "desktop";

export interface OriginMetadata {
  platform: Platform;
  app: string;
  collection: string;
  album?: string | null;
  device_name: string;
}

// Flattened state: `state` is the discriminant, `expires_at` only for temporary.
export interface MediaItem {
  id: string;
  name: string;
  size_bytes: number;
  mime: string;
  created_at: number;
  origin: OriginMetadata;
  state: "on_phone" | "on_tucklet" | "temporary";
  expires_at?: number | null;
  checksum?: string | null;
}

export interface StatusDto {
  free_bytes: number;
  total_bytes: number;
  item_count: number;
}

export interface EstimateDto {
  seconds: number;
  human: string;
  bytes_total: number;
  files: number;
}

export const api = {
  connect: (host: string, token: string) => invoke<void>("connect", { host, token }),
  status: () => invoke<StatusDto>("status"),
  library: () => invoke<MediaItem[]>("library"),
  thumbnailB64: (id: string) => invoke<string | null>("thumbnail_b64", { id }),
  pull: (id: string, out: string) => invoke<number>("pull", { id, out }),
  push: (file: string, item: MediaItem) => invoke<void>("push", { file, item }),
  remove: (id: string) => invoke<void>("delete", { id }),
  estimate: (ids: string[]) => invoke<EstimateDto>("estimate_ids", { ids }),
};

export function stateLabel(item: MediaItem): string {
  switch (item.state) {
    case "on_phone": return "On phone";
    case "on_tucklet": return "On Tucklet";
    case "temporary": return "Temporary";
  }
}

export function humanBytes(b: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = b, i = 0;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return i === 0 ? `${Math.round(v)} ${units[i]}` : `${v.toFixed(1)} ${units[i]}`;
}
