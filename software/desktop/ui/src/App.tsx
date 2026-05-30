// App.tsx — the Tucklet desktop GUI.
// Calm, plain-language UX matching the iOS/Android apps. Wired use needs no app
// (the device mounts as a USB drive); this is the wireless companion.
// License: PolyForm Noncommercial 1.0.0
import React, { useEffect, useMemo, useState } from "react";
import { api, MediaItem, StatusDto, EstimateDto, stateLabel, humanBytes } from "./api";

type Tab = "home" | "library" | "settings";

export default function App() {
  const [connected, setConnected] = useState(false);
  return connected ? <Main /> : <Connect onConnected={() => setConnected(true)} />;
}

function Connect({ onConnected }: { onConnected: () => void }) {
  // The BLE handshake (one-tap pairing) yields a host + token. Until that path
  // is wired into a command, accept them here so the app is usable today.
  const [host, setHost] = useState("192.168.4.1");
  const [token, setToken] = useState("");
  const [err, setErr] = useState<string | null>(null);

  async function go() {
    try { await api.connect(host, token); onConnected(); }
    catch (e) { setErr(String(e)); }
  }

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-paper p-8 text-ink">
      <div className="text-6xl text-accent">◈</div>
      <h1 className="text-2xl font-bold mt-4">Meet your Tucklet</h1>
      <p className="text-muted text-center max-w-sm mt-2">
        Bring it near and connect. Plugged in, it just appears as a drive — this
        app is for backing up and pulling photos wirelessly.
      </p>
      <div className="mt-6 w-full max-w-sm space-y-3">
        <input className="w-full border rounded-lg p-3" value={host}
          onChange={e => setHost(e.target.value)} placeholder="Tucklet address" />
        <input className="w-full border rounded-lg p-3" value={token}
          onChange={e => setToken(e.target.value)} placeholder="Session token" />
        <button onClick={go}
          className="w-full bg-accent text-white rounded-lg p-3 font-semibold">
          Connect my Tucklet
        </button>
        {err && <p className="text-red-600 text-sm">{err}</p>}
      </div>
    </div>
  );
}

function Main() {
  const [tab, setTab] = useState<Tab>("home");
  const [items, setItems] = useState<MediaItem[]>([]);
  const [status, setStatus] = useState<StatusDto | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());

  async function refresh() {
    try {
      setStatus(await api.status());
      setItems(await api.library());
    } catch { /* surface in a fuller build */ }
  }
  useEffect(() => { refresh(); }, []);

  return (
    <div className="min-h-screen bg-paper text-ink flex flex-col">
      <header className="flex items-center gap-6 px-6 py-3 border-b bg-white">
        <span className="font-bold text-accent">◈ Tucklet</span>
        {(["home", "library", "settings"] as Tab[]).map(t => (
          <button key={t} onClick={() => setTab(t)}
            className={tab === t ? "font-semibold text-ink" : "text-muted"}>
            {t[0].toUpperCase() + t.slice(1)}
          </button>
        ))}
      </header>
      <main className="flex-1 p-6 overflow-auto">
        {tab === "home" && <Home status={status} items={items} />}
        {tab === "library" && (
          <Library items={items} selected={selected} setSelected={setSelected} onChanged={refresh} />
        )}
        {tab === "settings" && <Settings status={status} />}
      </main>
    </div>
  );
}

function Home({ status, items }: { status: StatusDto | null; items: MediaItem[] }) {
  const onPhone = items.filter(i => i.state === "on_phone").length;
  return (
    <div className="space-y-4 max-w-2xl">
      <Card>
        <div className="text-center">
          <div className="text-4xl text-accent">◈</div>
          {status ? (
            <>
              <div className="font-bold">{humanBytes(status.free_bytes)} free of {humanBytes(status.total_bytes)}</div>
              <div className="text-muted text-sm">{status.item_count} items on your Tucklet</div>
            </>
          ) : <div className="text-muted">Connecting…</div>}
        </div>
      </Card>
      <Card>
        <div className="flex items-center gap-2">
          <span className="text-accent">{onPhone === 0 ? "✓" : "↑"}</span>
          <span>{onPhone === 0 ? "Everything's backed up" : `${onPhone} photos waiting to back up`}</span>
        </div>
      </Card>
    </div>
  );
}

function Library({ items, selected, setSelected, onChanged }: {
  items: MediaItem[]; selected: Set<string>; setSelected: (s: Set<string>) => void; onChanged: () => void;
}) {
  const [est, setEst] = useState<EstimateDto | null>(null);
  const groups = useMemo(() => {
    const m = new Map<string, MediaItem[]>();
    for (const it of items) { (m.get(it.origin.app) ?? m.set(it.origin.app, []).get(it.origin.app)!).push(it); }
    return [...m.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  }, [items]);

  function toggle(id: string) {
    const s = new Set(selected);
    s.has(id) ? s.delete(id) : s.add(id);
    setSelected(s);
  }

  async function estimateSel() { setEst(await api.estimate([...selected])); }
  async function getCopies() {
    for (const id of selected) await api.pull(id, `${id}`); // dest chosen by a real file dialog in production
    setSelected(new Set()); onChanged();
  }

  return (
    <div className="max-w-3xl">
      {selected.size > 0 && (
        <div className="flex items-center gap-3 mb-4">
          <button onClick={estimateSel} className="border rounded-lg px-3 py-2">Estimate</button>
          <button onClick={getCopies} className="bg-accent text-white rounded-lg px-3 py-2">Get a copy</button>
          {est && <span className="text-muted">About {est.human} · {est.files} items ({humanBytes(est.bytes_total)})</span>}
        </div>
      )}
      {groups.map(([app, group]) => (
        <div key={app} className="mb-4">
          <div className="text-muted font-semibold mb-1">{app}</div>
          {group.map(it => (
            <label key={it.id} className="flex items-center gap-3 p-2 bg-white rounded-lg mb-1 cursor-pointer">
              <input type="checkbox" checked={selected.has(it.id)} onChange={() => toggle(it.id)} />
              <Thumb id={it.id} state={it.state} />
              <div className="flex-1">
                <div>{it.name}</div>
                <div className="text-xs text-muted">{stateLabel(it)} · {humanBytes(it.size_bytes)}</div>
              </div>
            </label>
          ))}
        </div>
      ))}
    </div>
  );
}

function Thumb({ id, state }: { id: string; state: MediaItem["state"] }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    if (state !== "on_phone") api.thumbnailB64(id).then(b64 => { if (alive && b64) setSrc(`data:image/jpeg;base64,${b64}`); });
    return () => { alive = false; };
  }, [id, state]);
  return (
    <div className="w-11 h-11 rounded-lg bg-accent/10 flex items-center justify-center overflow-hidden">
      {src ? <img src={src} alt="" className="w-full h-full object-cover" /> : <span className="text-muted">▦</span>}
    </div>
  );
}

function Settings({ status }: { status: StatusDto | null }) {
  return (
    <div className="max-w-2xl space-y-4">
      <Card>
        <div className="font-semibold mb-1">Device</div>
        <div className="text-muted text-sm">
          {status ? `${humanBytes(status.free_bytes)} free of ${humanBytes(status.total_bytes)}` : "—"}
        </div>
      </Card>
      <Card>
        <div className="font-semibold mb-1">Plugged in</div>
        <div className="text-muted text-sm">
          When connected by cable, your Tucklet shows up as a normal drive in your
          file manager — no app needed. This app handles the wireless side.
        </div>
      </Card>
    </div>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return <div className="bg-white rounded-2xl p-5 shadow-sm">{children}</div>;
}
