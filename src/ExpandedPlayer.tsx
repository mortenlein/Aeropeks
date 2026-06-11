import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Icon } from "./icons";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { MediaInfo } from "./contracts";
import { HUE, T } from "./tokens";
import { Micro, Mono, SourceTag } from "./atoms";

function formatMs(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  return `${min}:${sec.toString().padStart(2, "0")}`;
}

function ExpandedPlayer() {
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
  const [albumArt, setAlbumArt] = useState<string>("");
  const [viewOffset, setViewOffset] = useState(0);
  const tickRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastThumbRef = useRef<string>("");

  const startTicker = (info: MediaInfo | null) => {
    if (tickRef.current) clearInterval(tickRef.current);
    if (!info?.is_playing) return;
    tickRef.current = setInterval(() => {
      setViewOffset((prev) => prev + 1000);
    }, 1000);
  };

  const loadAlbumArt = async (thumb: string) => {
    if (!thumb || thumb === lastThumbRef.current) return;
    lastThumbRef.current = thumb;
    try {
      const b64 = await invoke<string>("get_album_art", { thumb });
      if (b64) setAlbumArt(`data:image/jpeg;base64,${b64}`);
      else setAlbumArt("");
    } catch {
      setAlbumArt("");
    }
  };

  useEffect(() => {
    invoke<MediaInfo | null>("get_media_info_unified")
      .then((info) => {
        setMediaInfo(info);
        setViewOffset(info?.view_offset_ms ?? 0);
        startTicker(info);
        loadAlbumArt(info?.thumbnail ?? "");
      })
      .catch(() => {});

    const unlistenMedia = listen<MediaInfo | null>("media-change", (event) => {
      const info = event.payload;
      setMediaInfo(info);
      if (info) {
        setViewOffset(info.view_offset_ms);
        startTicker(info);
        loadAlbumArt(info.thumbnail ?? "");
      } else {
        if (tickRef.current) clearInterval(tickRef.current);
      }
    });

    return () => {
      if (tickRef.current) clearInterval(tickRef.current);
      unlistenMedia.then((f) => f());
    };
  }, []);

  const handleMediaControl = async (action: string) => {
    if (!mediaInfo) return;
    try {
      await invoke("media_control_unified", { action });
      if (action === "play_pause") {
        const nowPlaying = !mediaInfo.is_playing;
        const updated = { ...mediaInfo, is_playing: nowPlaying };
        setMediaInfo(updated);
        startTicker(updated);
      }
    } catch (e) {
      console.error("Plex Control Error:", e);
    }
  };

  const duration = mediaInfo?.duration_ms || 1;
  const progressPct = Math.min((viewOffset / duration) * 100, 100);

  return (
    <div
      onContextMenu={(e) => e.preventDefault()}
      style={{ width: "100%", height: "100%", background: "transparent", display: "flex" }}
    >
      <div
        style={{
          flex: 1,
          background: T.panelBg,
          borderRadius: T.panelR,
          border: T.panelBorder,
          boxShadow: T.shadow,
          padding: T.pad,
          display: "flex",
          flexDirection: "column",
          gap: 12,
          position: "relative",
          overflow: "hidden",
        }}
      >
        {/* Drag region */}
        <div
          data-tauri-drag-region
          style={{ position: "absolute", top: 0, left: 0, right: 0, height: 28, cursor: "grab", zIndex: 1 }}
        />

        {/* Header: NOW PLAYING · PLEX tag · ✕ */}
        <div style={{ display: "flex", alignItems: "center", paddingBottom: 9, borderBottom: `1px solid ${T.divider}`, marginBottom: 3 }}>
          <Micro color={HUE.media} style={{ flex: 1 }}>Now Playing</Micro>
          <SourceTag name="PLEX" />
          <button
            onClick={() => getCurrentWindow().hide()}
            style={{ background: "none", border: "none", cursor: "pointer", color: T.t3, display: "flex", padding: 0, marginLeft: 8 }}
          >
            <Icon name="close" size={11} />
          </button>
        </div>

        {/* Body: [art] [text+progress flex-1] [transport] */}
        <div style={{ display: "flex", alignItems: "center", gap: 14 }}>

          {/* Album art — 92×92, 5px radius */}
          <div
            style={{
              width: 92, height: 92, flexShrink: 0,
              borderRadius: T.cardR,
              background: T.cardBg,
              border: T.cardBorder,
              overflow: "hidden",
              display: "flex", alignItems: "center", justifyContent: "center",
              color: T.t3,
            }}
          >
            {albumArt ? (
              <img src={albumArt} alt="Album Art" style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />
            ) : (
              <Icon name="music" size={24} />
            )}
          </div>

          {/* Text column + progress */}
          <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 8 }}>
            <div>
              <div style={{ fontSize: 16, fontWeight: 700, color: T.t1, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", letterSpacing: "-0.01em" }}>
                {mediaInfo?.title || "No Media Playing"}
              </div>
              <div style={{ fontFamily: "var(--font-ui)", fontSize: 12.5, color: T.t2, marginTop: 3 }}>
                {mediaInfo?.artist || "Syncing playback..."}
              </div>
              {mediaInfo?.album && (
                <Mono size={9} color={T.t3} style={{ marginTop: 3, textTransform: "uppercase", letterSpacing: "0.1em" }}>
                  {mediaInfo.album}
                </Mono>
              )}
            </div>

            {/* 3px progress bar — no knob */}
            <div>
              <div style={{ height: 3, background: T.divider, borderRadius: 2, overflow: "hidden", marginBottom: 5 }}>
                <div style={{ width: `${progressPct}%`, height: "100%", background: HUE.media, borderRadius: 2, transition: "width 0.5s linear" }} />
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <Mono size={9.5} color={T.t3}>{formatMs(viewOffset)}</Mono>
                <Mono size={9.5} color={T.t3}>{formatMs(duration)}</Mono>
              </div>
            </div>
          </div>

          {/* Transport cluster — prev / play-pause / next */}
          <div style={{ display: "flex", alignItems: "center", gap: 4, flexShrink: 0 }}>
            <button
              onClick={() => handleMediaControl("previous")}
              style={{ background: "none", border: "none", cursor: "pointer", color: T.t2, display: "flex", padding: 5, borderRadius: T.ctlR }}
            >
              <Icon name="prev" size={14} />
            </button>
            <button
              onClick={() => handleMediaControl("play_pause")}
              style={{
                width: 32, height: 32, borderRadius: T.ctlR,
                background: HUE.media, border: "none",
                display: "flex", alignItems: "center", justifyContent: "center",
                cursor: "pointer", color: "#10131a", flexShrink: 0,
              }}
            >
              {mediaInfo?.is_playing ? <Icon name="pause" size={14} /> : <Icon name="play" size={14} />}
            </button>
            <button
              onClick={() => handleMediaControl("next")}
              style={{ background: "none", border: "none", cursor: "pointer", color: T.t2, display: "flex", padding: 5, borderRadius: T.ctlR }}
            >
              <Icon name="next" size={14} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ExpandedPlayer;
