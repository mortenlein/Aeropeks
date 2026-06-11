import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Panel, Mono } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import type { PinnedShortcut } from "./contracts";

const MAX_SLOTS = 8;

export function parseShortcutUrl(raw: string): string | null {
  let v = raw.trim();
  if (!v) return null;
  if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(v)) v = "https://" + v;
  try {
    const u = new URL(v);
    if (u.protocol !== "http:" && u.protocol !== "https:") return null;
    if (!u.hostname.includes(".")) return null;
    return u.href;
  } catch {
    return null;
  }
}

export function shortcutHost(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

// host → data URI, or null when the fetch failed (monogram fallback).
const faviconCache = new Map<string, string | null>();

function Favicon({ url, size = 14 }: { url: string; size?: number }) {
  const host = shortcutHost(url);
  // undefined = resolving, null = fallback monogram, string = icon data URI
  const [src, setSrc] = useState<string | null | undefined>(faviconCache.get(host));

  useEffect(() => {
    if (faviconCache.has(host)) {
      setSrc(faviconCache.get(host));
      return;
    }
    let disposed = false;
    setSrc(undefined);
    invoke<string>("get_favicon", { url })
      .then((uri) => {
        faviconCache.set(host, uri);
        if (!disposed) setSrc(uri);
      })
      .catch(() => {
        faviconCache.set(host, null);
        if (!disposed) setSrc(null);
      });
    return () => {
      disposed = true;
    };
  }, [host, url]);

  if (src) {
    return <img src={src} width={size} height={size} alt="" draggable={false} style={{ borderRadius: 3, flexShrink: 0 }} />;
  }
  return (
    <span style={{ width: size, height: size, borderRadius: 3, background: 'rgba(255,255,255,0.08)', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0, animation: src === undefined ? 'aeroPulse 1.4s ease-in-out infinite' : 'none' }}>
      {src === null && (
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: Math.round(size * 0.58), fontWeight: 600, color: T.t2, textTransform: 'uppercase', lineHeight: 1 }}>
          {host.charAt(0)}
        </span>
      )}
    </span>
  );
}

export function ShortcutsPanel({ shortcuts, onClose }: {
  shortcuts: PinnedShortcut[];
  onClose?: () => void;
}) {
  const [draft, setDraft] = useState("");
  const [draftName, setDraftName] = useState("");
  const [hoveredRow, setHoveredRow] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editUrl, setEditUrl] = useState("");
  const valid = parseShortcutUrl(draft);
  const full = shortcuts.length >= MAX_SLOTS;

  const save = (next: PinnedShortcut[]) =>
    invoke("set_pinned_shortcuts", { shortcuts: next }).catch((e) =>
      console.error("Saving shortcuts failed", e),
    );
  const add = () => {
    if (!valid || full) return;
    save([...shortcuts, { id: `sc-${Date.now()}`, url: valid, name: draftName.trim().slice(0, 60) }]);
    setDraft("");
    setDraftName("");
  };
  const openShortcut = (id: string) => {
    invoke("open_shortcut", { id }).catch((e) => console.error("Opening shortcut failed", e));
    onClose?.();
  };
  const startEdit = (s: PinnedShortcut) => {
    setEditingId(s.id);
    setEditName(s.name);
    setEditUrl(s.url);
  };
  const editValid = parseShortcutUrl(editUrl);
  const commitEdit = () => {
    if (!editingId || !editValid) return;
    save(shortcuts.map((s) =>
      s.id === editingId ? { ...s, name: editName.trim().slice(0, 60), url: editValid } : s,
    ));
    setEditingId(null);
  };

  const editInputStyle = {
    background: T.inputBg,
    border: T.inputBorder,
    borderRadius: T.ctlR,
    padding: '3px 7px',
    outline: 'none',
    color: T.t1,
    caretColor: 'var(--accent)',
    minWidth: 0,
  } as const;

  return (
    <Panel
      w={340}
      title="Shortcuts"
      icon={<Icon name="extlink" size={13} />}
      hue="var(--accent)"
      actions={<Mono size={9.5} color={full ? HUE.amber : T.t3}>{shortcuts.length}/{MAX_SLOTS}</Mono>}
    >
      {/* add row */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12, padding: '7px 10px', background: T.inputBg, border: T.inputBorder, borderRadius: T.ctlR }}>
        {valid
          ? <Favicon url={valid} size={14} />
          : <span style={{ width: 14, height: 14, borderRadius: 3, border: `1px dashed ${T.divider}`, flexShrink: 0 }} />}
        <input
          value={draftName}
          onChange={(e) => setDraftName(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') add(); }}
          placeholder="Name"
          disabled={full}
          maxLength={60}
          style={{ width: 76, flexShrink: 0, background: 'transparent', border: 'none', outline: 'none', fontFamily: 'var(--font-ui)', fontSize: 12, fontWeight: 600, color: T.t1, caretColor: 'var(--accent)' }}
        />
        <span style={{ width: 1, alignSelf: 'stretch', background: T.divider, flexShrink: 0 }} />
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') add(); }}
          placeholder={full ? 'all slots in use' : 'paste a URL'}
          disabled={full}
          style={{ flex: 1, minWidth: 0, background: 'transparent', border: 'none', outline: 'none', fontFamily: 'var(--font-mono)', fontSize: 11, color: T.t1, caretColor: 'var(--accent)' }}
        />
        <span
          onClick={add}
          style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, letterSpacing: '0.12em', textTransform: 'uppercase', fontWeight: 600, color: valid && !full ? 'var(--accent)' : T.t3, cursor: valid && !full ? 'pointer' : 'default', userSelect: 'none' }}
        >
          Add
        </span>
      </div>

      {/* list */}
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        {shortcuts.map((s, i) =>
          editingId === s.id ? (
            <div
              key={s.id}
              style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '9px 6px', borderTop: i > 0 ? `1px solid ${T.divider}` : 'none' }}
            >
              <Favicon url={s.url} size={16} />
              <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', gap: 4 }}>
                <input
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') setEditingId(null); }}
                  placeholder="Name"
                  maxLength={60}
                  autoFocus
                  style={{ ...editInputStyle, fontFamily: 'var(--font-ui)', fontSize: 12, fontWeight: 600 }}
                />
                <input
                  value={editUrl}
                  onChange={(e) => setEditUrl(e.target.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') setEditingId(null); }}
                  placeholder="URL"
                  style={{ ...editInputStyle, fontFamily: 'var(--font-mono)', fontSize: 10.5 }}
                />
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6, alignItems: 'flex-end' }}>
                <span
                  onClick={commitEdit}
                  style={{ fontFamily: 'var(--font-mono)', fontSize: 9, letterSpacing: '0.12em', textTransform: 'uppercase', fontWeight: 600, color: editValid ? 'var(--accent)' : T.t3, cursor: editValid ? 'pointer' : 'default', userSelect: 'none' }}
                >
                  Save
                </span>
                <span
                  onClick={() => setEditingId(null)}
                  style={{ fontFamily: 'var(--font-mono)', fontSize: 9, letterSpacing: '0.12em', textTransform: 'uppercase', fontWeight: 600, color: T.t3, cursor: 'pointer', userSelect: 'none' }}
                >
                  Cancel
                </span>
              </div>
            </div>
          ) : (
            <div
              key={s.id}
              onClick={() => openShortcut(s.id)}
              onMouseEnter={() => setHoveredRow(s.id)}
              onMouseLeave={() => setHoveredRow(null)}
              style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '9px 6px', borderRadius: T.ctlR, cursor: 'pointer', background: hoveredRow === s.id ? T.ctlBg : 'transparent', borderTop: i > 0 ? `1px solid ${T.divider}` : 'none' }}
            >
              <Favicon url={s.url} size={16} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 12.5, fontWeight: 600, color: T.t1, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                  {s.name || shortcutHost(s.url)}
                </div>
                <Mono size={9.5} color={T.t3} style={{ display: 'block', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{s.url}</Mono>
              </div>
              <span
                onClick={(e) => {
                  e.stopPropagation();
                  startEdit(s);
                }}
                title="Edit"
                style={{ color: T.t3, display: 'flex', cursor: 'pointer', padding: 2, visibility: hoveredRow === s.id ? 'visible' : 'hidden' }}
              >
                <Icon name="gear" size={11} />
              </span>
              <span
                onClick={(e) => {
                  e.stopPropagation();
                  save(shortcuts.filter((x) => x.id !== s.id));
                }}
                title="Remove"
                style={{ color: T.t3, display: 'flex', cursor: 'pointer', padding: 2, visibility: hoveredRow === s.id ? 'visible' : 'hidden' }}
              >
                <Icon name="close" size={9} />
              </span>
            </div>
          ),
        )}
        {shortcuts.length === 0 && (
          <div style={{ padding: '18px 2px', fontSize: 11.5, color: T.t3 }}>
            No shortcuts yet — paste a URL above to pin one.
          </div>
        )}
      </div>

      <div style={{ fontSize: 10.5, color: T.t3, lineHeight: 1.55, marginTop: 10, paddingTop: 9, borderTop: `1px solid ${T.divider}` }}>
        Icons are fetched from each site's favicon. While resolving, a pulsing slot holds the space;
        if a site has none, a monogram of its first letter steps in.
      </div>
    </Panel>
  );
}
