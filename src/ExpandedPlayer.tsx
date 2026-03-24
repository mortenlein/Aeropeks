import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Play, Pause, SkipBack, SkipForward, Music, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface MediaInfo {
  title: string;
  artist: string;
  album: string;
  thumb: string;
  duration_ms: number;
  view_offset_ms: number;
  is_playing: boolean;
  session_id: string;
  machine_id: string;
  address: string;
}

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
    invoke<MediaInfo>("get_media_info")
      .then((info) => {
        setMediaInfo(info);
        setViewOffset(info.view_offset_ms);
        startTicker(info);
        loadAlbumArt(info.thumb);
      })
      .catch(() => {});

    const unlistenMedia = listen<MediaInfo | null>("media-change", (event) => {
      const info = event.payload;
      setMediaInfo(info);
      if (info) {
        setViewOffset(info.view_offset_ms);
        startTicker(info);
        loadAlbumArt(info.thumb);
      } else {
        if (tickRef.current) clearInterval(tickRef.current);
      }
    });

    return () => {
      if (tickRef.current) clearInterval(tickRef.current);
      unlistenMedia.then((f) => f());
    };
  }, []);

  const handlePlexControl = async (command: string) => {
    if (!mediaInfo?.session_id || !mediaInfo?.machine_id) return;
    try {
      await invoke("plex_control", {
        command,
        sessionId: mediaInfo.session_id,
        machineId: mediaInfo.machine_id,
        address: mediaInfo.address,
      });
      // Optimistically update play state so the UI doesn't lag
      if (command === "play" || command === "pause") {
        const nowPlaying = command === "play";
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
    <div className="expanded-outer" onContextMenu={(e) => e.preventDefault()}>
      <div className="expanded-player">
        <button className="close-btn" onClick={() => getCurrentWindow().hide()}>
          <X size={14} />
        </button>

        <div className="album-art-container">
          <div className="album-art">
            {albumArt ? (
              <img src={albumArt} alt="Album Art" className="album-art-img" />
            ) : (
              <Music size={32} color="var(--accent)" />
            )}
          </div>
        </div>

        <div className="player-content">
          <div className="track-info">
            <h2 className="title">{mediaInfo?.title || "No Media Playing"}</h2>
            <p className="artist">{mediaInfo?.artist || "Syncing playback..."}</p>
            {mediaInfo?.album && <p className="album">{mediaInfo.album}</p>}
          </div>

          <div className="progress-container">
            <div className="progress-bar">
              <div className="progress-fill" style={{ width: `${progressPct}%` }}>
                <div className="progress-knob" />
              </div>
            </div>
            <div className="time-info">
              <span>{formatMs(viewOffset)}</span>
              <span>{formatMs(duration)}</span>
            </div>
          </div>

          <div className="player-controls-expanded">
            <button className="ctrl-btn" onClick={() => handlePlexControl("prev")}>
              <SkipBack size={20} fill="currentColor" />
            </button>
            <button
              className="play-btn"
              onClick={() => handlePlexControl(mediaInfo?.is_playing ? "pause" : "play")}
            >
              {mediaInfo?.is_playing ? (
                <Pause size={24} fill="white" />
              ) : (
                <Play size={24} fill="white" />
              )}
            </button>
            <button className="ctrl-btn" onClick={() => handlePlexControl("next")}>
              <SkipForward size={20} fill="currentColor" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ExpandedPlayer;
