import { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Music, Volume2, Clock, Play, Pause, SkipBack, SkipForward, Terminal as TerminalIcon, Bluetooth, Battery, Mic, MicOff, Video, Cloud, Power, Settings as SettingsIcon, Sun, CloudRain, CloudLightning, Snowflake, Wind, CloudSun, CloudDrizzle } from "lucide-react";
import { WeatherPopover } from "./WeatherPopover";

interface TerminalShortcut {
  id: string;
  label: string;
  cmd: string;
}

interface AppSettings {
  plex_url: string;
  plex_token: string;
  accent_color: string;
  terminal_shortcuts: TerminalShortcut[];
  weather_location: string;
  weather_lat: number | null;
  weather_lon: number | null;
  use_24h: boolean;
}

interface MediaInfo {
  title: string;
  artist: string;
  album: string;
  is_playing: boolean;
  thumbnail?: string;
  duration_ms: number;
  view_offset_ms: number;
  source: string;
  session_id?: string;
  machine_id?: string;
  address?: string;
}

interface BatteryStatus {
  percentage: number;
  is_charging: boolean;
  has_battery: boolean;
}

interface HourlyForecast {
  time: string;
  temp: number;
  symbol: string;
  precip: number;
}

interface DailyForecast {
  date: string;
  temp_min: number;
  temp_max: number;
  symbol: string;
}

interface WeatherDetailed {
  temp: number;
  symbol: string;
  precip: number;
  place_name: string;
  hourly: HourlyForecast[];
  daily: DailyForecast[];
}

interface ObsStatus {
  is_streaming: boolean;
  is_recording: boolean;
}

function App() {
  const [windowTitle, setWindowTitle] = useState("Aeropeks");
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
  const [volume, setVolume] = useState(0.5);
  const [showVolume, setShowVolume] = useState(false);
  const [battery, setBattery] = useState<BatteryStatus | null>(null);
  const [bluetooth, setBluetooth] = useState(false);
  const [micMuted, setMicMuted] = useState(false);
  const [obsStatus, setObsStatus] = useState<ObsStatus | null>(null);
  const [weather, setWeather] = useState<WeatherDetailed | null>(null);
  const [showWeather, setShowWeather] = useState(false);
  const weatherRef = useRef<HTMLDivElement>(null);
  
  const [showPowerMenu, setShowPowerMenu] = useState(false);
  const powerMenuRef = useRef<HTMLDivElement>(null);
  const [time, setTime] = useState("");
  const [use24h, setUse24h] = useState(true);

  const volumeRef = useRef<HTMLDivElement>(null);

  const formatTime = (date: Date, is24h: boolean) => {
    return date.toLocaleTimeString(undefined, { 
      hour: '2-digit', 
      minute: '2-digit',
      hour12: !is24h
    });
  };

  const fetchMedia = async () => {
    try {
      const info = await invoke<MediaInfo | null>("get_media_info_unified");
      setMediaInfo(info);
    } catch (e) {
      setMediaInfo(null);
    }
  };

  const handleMediaControl = async (action: string) => {
    try {
      await invoke("media_control_unified", { action, media: mediaInfo });
    } catch (e) {
      console.error("Media control failed", e);
    }
  };

  const fetchStatuses = async () => {
    try {
      setBattery(await invoke<BatteryStatus>("get_battery_status"));
      setBluetooth(await invoke<boolean>("get_bluetooth_status"));
      setMicMuted(await invoke<boolean>("get_mic_status"));
      setObsStatus(await invoke<ObsStatus>("get_obs_status"));
    } catch (e) {}
  };

  const fetchWeather = async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      if (s.weather_lat && s.weather_lon) {
        const w = await invoke<WeatherDetailed>("get_weather", { 
          lat: s.weather_lat, 
          lon: s.weather_lon,
          placeName: s.weather_location || "Unknown"
        });
        setWeather(w);
      }
    } catch (e) {
      console.error("Weather fetch failed:", e);
    }
  };

  useEffect(() => {
    invoke<AppSettings>("get_settings").then((s) => {
      if (s.accent_color) {
        document.documentElement.style.setProperty("--accent", s.accent_color);
      }
      setUse24h(s.use_24h !== false);
      setTime(formatTime(new Date(), s.use_24h !== false));
    }).catch(() => {});
    invoke<number>("get_volume").then(setVolume);
    fetchMedia();
    fetchStatuses();
    fetchWeather();

    const pollInterval = setInterval(fetchMedia, 30000);
    const statusInterval = setInterval(fetchStatuses, 5000);
    const weatherInterval = setInterval(fetchWeather, 600000); // 10 mins
    const timeInterval = setInterval(() => {
      setTime(formatTime(new Date(), use24h));
    }, 10000);

    const unlistenWindow = listen<string>("window-change", (event) => {
      setWindowTitle(event.payload || "Desktop");
    });

    const unlistenMedia = listen<MediaInfo | null>("media-change", (event) => {
      if (event.payload) {
        setMediaInfo(event.payload);
      } else {
        setMediaInfo(null);
      }
    });

    const unlistenSettings = listen<AppSettings>("settings-changed", (event) => {
      setUse24h(event.payload.use_24h !== false);
      if (event.payload.accent_color) {
        document.documentElement.style.setProperty("--accent", event.payload.accent_color);
      }
    });

    return () => {
      clearInterval(pollInterval);
      clearInterval(statusInterval);
      clearInterval(weatherInterval);
      clearInterval(timeInterval);
      unlistenWindow.then(f => f());
      unlistenMedia.then(f => f());
      unlistenSettings.then(f => f());
    };
  }, []);

  // Click-away listener
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (volumeRef.current && !volumeRef.current.contains(event.target as Node)) {
        setShowVolume(false);
      }
      if (powerMenuRef.current && !powerMenuRef.current.contains(event.target as Node)) {
        setShowPowerMenu(false);
      }
      if (weatherRef.current && !weatherRef.current.contains(event.target as Node)) {
        setShowWeather(false);
      }
    };

    if (showVolume || showPowerMenu || showWeather) {
      document.addEventListener("mousedown", handleClickOutside);
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [showVolume, showPowerMenu, showWeather]);

  useEffect(() => {
    // Dynamic window expansion for dropdowns/popovers
    if (showVolume || showPowerMenu || showWeather) {
       invoke("set_window_height", { height: 600 }).catch(console.error);
    } else {
       invoke("set_window_height", { height: 32 }).catch(console.error);
    }
  }, [showVolume, showPowerMenu, showWeather]);

  const handleVolumeChange = (newVol: number) => {
    setVolume(newVol);
    invoke("set_volume", { volume: newVol });
  };


  const handleMicToggle = async () => {
    try {
      const isMuted = await invoke<boolean>("toggle_mic_mute");
      setMicMuted(isMuted);
    } catch (e) {}
  };

  return (
    <div className="main-window">
      <div className="menu-bar" onContextMenu={(e) => e.preventDefault()}>
        <div className="left-section">
          <div className="app-icon" />
          <div className="window-title">
            {windowTitle}
          </div>
        </div>
        
        {/* ... existing App content ... */}
        
        <div className="center-section">
          {mediaInfo ? (
            <div className={`media-info ${!mediaInfo.is_playing ? 'paused' : ''}`} onClick={() => invoke("toggle_expanded_player")}>
               <div className="media-controls">
                  <div className="control-node" title="Previous" onClick={(e) => { e.stopPropagation(); handleMediaControl("previous"); }}><SkipBack size={14} /></div>
                  <div className="control-node" title={mediaInfo.is_playing ? "Pause" : "Play"} onClick={(e) => { e.stopPropagation(); handleMediaControl("play_pause"); }}>
                    {mediaInfo.is_playing ? <Pause size={14} fill="currentColor" /> : <Play size={14} fill="currentColor" />}
                  </div>
                  <div className="control-node" title="Next" onClick={(e) => { e.stopPropagation(); handleMediaControl("next"); }}><SkipForward size={14} /></div>
               </div>
               <Music size={14} className={mediaInfo.is_playing ? "playing-icon" : ""} />
               <div className="media-text">
                 <span className="artist">{mediaInfo.artist}</span>
                 <span className="separator">•</span>
                 <span className="title">{mediaInfo.title}</span>
               </div>
            </div>
          ) : (
            <div className="media-info" style={{ opacity: 0.5 }}>
              <Music size={14} />
              <span>Nothing Playing</span>
            </div>
          )}
        </div>

        <div className="right-section">
          {weather && (
            <div 
              className="status-item" 
              onClick={() => setShowWeather(!showWeather)}
              ref={weatherRef}
            >
              <Cloud size={14} />
              <span>{Math.round(weather.temp)}°C</span>
              
              {showWeather && (
                <div onClick={(e) => e.stopPropagation()}>
                  <WeatherPopover 
                    data={weather} 
                    onClose={() => setShowWeather(false)} 
                  />
                </div>
              )}
            </div>
          )}
          
          {obsStatus && (obsStatus.is_recording || obsStatus.is_streaming) && (
            <div className="status-item obs-active" title={obsStatus.is_streaming ? "OBS Streaming" : "OBS Recording"}>
              <Video size={16} />
            </div>
          )}

          <div className={`status-item ${bluetooth ? 'active' : 'inactive'}`} title="Bluetooth">
            <Bluetooth size={16} />
          </div>

          <div className="status-item" onClick={handleMicToggle} title={micMuted ? "Unmute Mic" : "Mute Mic"}>
            {micMuted ? <MicOff size={16} color="#ef4444" /> : <Mic size={16} />}
          </div>

          <div 
            className="status-item volume-item"
            ref={volumeRef}
            onClick={() => {
              setShowVolume(!showVolume);
              setShowPowerMenu(false);
            }}
            title="Volume"
          >
            <Volume2 size={16} />
            {showVolume && (
              <div className="volume-popover" onClick={(e) => e.stopPropagation()}>
                <input 
                  type="range" 
                  min="0" 
                  max="1" 
                  step="0.01" 
                  value={volume} 
                  onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
                  className="volume-slider-vertical"
                  autoFocus
                />
              </div>
            )}
          </div>

          {battery && battery.has_battery && (
            <div className={`status-item ${battery.is_charging ? 'charging' : ''}`} title={`Battery: ${battery.percentage}%`}>
              <Battery size={16} />
              <span style={{ fontSize: '10px' }}>{battery.percentage}%</span>
            </div>
          )}

          <div className="status-item" onClick={() => invoke("toggle_terminal_panel")} onContextMenu={(e) => { e.preventDefault(); invoke("show_terminal_context_menu"); }}>
            <TerminalIcon size={16} />
          </div>

          <div 
            className="status-item" 
            ref={powerMenuRef}
            onClick={() => {
              setShowPowerMenu(!showPowerMenu);
              setShowVolume(false);
            }}
          >
            <Power size={16} />
            {showPowerMenu && (
              <div className="power-menu dropdown" onClick={(e) => e.stopPropagation()}>
                <div onClick={() => invoke("system_power_action", { action: "lock" })}>Lock</div>
                <div onClick={() => invoke("system_power_action", { action: "sleep" })}>Sleep</div>
                <div onClick={() => invoke("system_power_action", { action: "restart" })}>Restart</div>
                <div onClick={() => invoke("system_power_action", { action: "shutdown" })} className="danger">Shut Down</div>
              </div>
            )}
          </div>

          <div className="status-item" onClick={() => invoke("open_settings")}>
            <SettingsIcon size={16} />
          </div>

          <div className="status-item">
            <Clock size={16} />
            <span>{time}</span>
          </div>
        </div>
      </div>
    </div>
  );
}


export default App;
