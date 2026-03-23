import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Music, Volume2, Clock, Play, Pause, SkipBack, SkipForward } from "lucide-react";

interface MediaInfo {
  title: string;
  artist: string;
  is_playing: boolean;
  session_id: string;
  machine_id: string;
  address: string;
}

function App() {
  const [windowTitle, setWindowTitle] = useState("Aeropeks");
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
  const [volume, setVolume] = useState(0.5);
  const [showVolume, setShowVolume] = useState(false);
  const [time, setTime] = useState(new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));
  const [updateCount, setUpdateCount] = useState(0);
  const [lastCommand, setLastCommand] = useState<string>("");

  const fetchMedia = () => {
    invoke<MediaInfo>("get_media_info").then((info) => {
      if (info && info.title !== "Nothing Playing") {
        setMediaInfo(info);
      }
    }).catch(() => {});
  };

  useEffect(() => {
    invoke<number>("get_volume").then(setVolume);
    fetchMedia();

    const pollInterval = setInterval(fetchMedia, 5000);
    const timeInterval = setInterval(() => {
      setTime(new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));
    }, 60000);

    const unlistenWindow = listen<string>("window-change", (event) => {
      setWindowTitle(event.payload || "Desktop");
    });

    const unlistenMedia = listen<MediaInfo | null>("media-change", (event) => {
      if (event.payload) {
        setMediaInfo(event.payload);
        setUpdateCount(prev => prev + 1);
      } else {
        setMediaInfo(null);
      }
    });

    return () => {
      clearInterval(pollInterval);
      clearInterval(timeInterval);
      unlistenWindow.then(f => f());
      unlistenMedia.then(f => f());
    };
  }, []);

  const handleVolumeChange = (newVol: number) => {
    setVolume(newVol);
    invoke("set_volume", { volume: newVol });
  };

  const handlePlexControl = async (command: string) => {
    if (!mediaInfo?.session_id || !mediaInfo?.machine_id) {
      setLastCommand("No Session");
      return;
    }
    setLastCommand(command + "...");
    try {
      await invoke("plex_control", { 
        command, 
        sessionId: mediaInfo.session_id, 
        machineId: mediaInfo.machine_id,
        address: mediaInfo.address
      });
      setLastCommand(command + " OK");
    } catch (e) {
      console.error("Plex Control Error:", e);
      setLastCommand("Error");
    }
  };

  return (
    <div className="menu-bar" onContextMenu={(e) => e.preventDefault()}>
      <div className="left-section">
        <div className="app-icon" />
        <div className="window-title">
          {windowTitle}
        </div>
      </div>

      <div className="center-section">
        {mediaInfo ? (
          <div className={`media-info ${!mediaInfo.is_playing ? 'paused' : ''}`}>
             <div className="media-controls">
                <div className="control-node" onClick={() => handlePlexControl("prev")}><SkipBack size={16} /></div>
                <div className="control-node" onClick={() => handlePlexControl(mediaInfo.is_playing ? "pause" : "play")}>
                  {mediaInfo.is_playing ? <Pause size={16} /> : <Play size={16} />}
                </div>
                <div className="control-node" onClick={() => handlePlexControl("next")}><SkipForward size={16} /></div>
             </div>
             <Music size={14} className={mediaInfo.is_playing ? "playing-icon" : ""} />
             <span>{mediaInfo.artist} - {mediaInfo.title}</span>
          </div>
        ) : (
          <div className="media-info" style={{ opacity: 0.5 }}>
            <Music size={14} />
            <span>Nothing Playing</span>
          </div>
        )}
      </div>

      <div className="right-section">
        <div 
          className="status-item"
          onMouseEnter={() => setShowVolume(true)}
          onMouseLeave={() => setShowVolume(false)}
        >
          <Volume2 size={16} />
          {showVolume && (
            <input 
              type="range" 
              min="0" 
              max="1" 
              step="0.01" 
              value={volume} 
              onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
              className="volume-slider"
            />
          )}
        </div>
        <div className="status-item">
          <Clock size={16} />
          <span>{time}</span>
        </div>
      </div>
    </div>
  );
}

export default App;
