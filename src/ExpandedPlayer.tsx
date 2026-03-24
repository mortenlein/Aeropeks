import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Play, Pause, SkipBack, SkipForward, Music, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface MediaInfo {
  title: string;
  artist: string;
  is_playing: boolean;
  session_id: string;
  machine_id: string;
  address: string;
}

function ExpandedPlayer() {
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);

  useEffect(() => {
    invoke<MediaInfo>("get_media_info").then(setMediaInfo).catch(() => {});

    const unlistenMedia = listen<MediaInfo | null>("media-change", (event) => {
      setMediaInfo(event.payload);
    });

    return () => {
      unlistenMedia.then(f => f());
    };
  }, []);

  const handlePlexControl = async (command: string) => {
    if (!mediaInfo?.session_id || !mediaInfo?.machine_id) return;
    try {
      await invoke("plex_control", { 
        command, 
        sessionId: mediaInfo.session_id, 
        machineId: mediaInfo.machine_id,
        address: mediaInfo.address
      });
    } catch (e) {
      console.error("Plex Control Error:", e);
    }
  };

  return (
    <div className="expanded-outer" onContextMenu={(e) => e.preventDefault()}>
      <div className="expanded-player">
        <button className="close-btn" onClick={() => getCurrentWindow().hide()}>
          <X size={14} />
        </button>
        
        <div className="album-art-container">
          <div className="album-art">
             <Music size={32} color="var(--accent)" />
          </div>
        </div>

        <div className="player-content">
          <div className="track-info">
            <h2 className="title">{mediaInfo?.title || "No Media Playing"}</h2>
            <p className="artist">{mediaInfo?.artist || "Syncing playback..."}</p>
          </div>

          <div className="progress-container">
            <div className="progress-bar">
              <div className="progress-fill" style={{ width: mediaInfo ? "45%" : "0%" }}>
                <div className="progress-knob" />
              </div>
            </div>
            <div className="time-info">
              <span>{mediaInfo ? "1:42" : "0:00"}</span>
              <span>{mediaInfo ? "3:54" : "0:00"}</span>
            </div>
          </div>

          <div className="player-controls-expanded">
            <button className="ctrl-btn" onClick={() => handlePlexControl("prev")}>
              <SkipBack size={20} fill="currentColor" />
            </button>
            <button className="play-btn" onClick={() => handlePlexControl(mediaInfo?.is_playing ? "pause" : "play")}>
              {mediaInfo?.is_playing ? <Pause size={24} fill="white" /> : <Play size={24} fill="white" />}
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
