import { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Music, Volume2, Clock, Play, Pause, SkipBack, SkipForward, Terminal as TerminalIcon, Bluetooth, Battery, Mic, MicOff, Video, Cloud, Power, Settings as SettingsIcon, Shield } from "lucide-react";
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

interface BluetoothStatus {
  connected: boolean;
  devices: string[];
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
  const [bluetooth, setBluetooth] = useState<BluetoothStatus>({ connected: false, devices: [] });
  const [showBluetoothMenu, setShowBluetoothMenu] = useState(false);
  const bluetoothRef = useRef<HTMLDivElement>(null);
  const [micMuted, setMicMuted] = useState(false);
  const [privacyMode, setPrivacyMode] = useState(false);
  const [obsStatus, setObsStatus] = useState<ObsStatus | null>(null);
  const [weather, setWeather] = useState<WeatherDetailed | null>(null);
  const [showWeather, setShowWeather] = useState(false);
  const weatherRef = useRef<HTMLDivElement>(null);
  
  const [showPowerMenu, setShowPowerMenu] = useState(false);
  const powerMenuRef = useRef<HTMLDivElement>(null);
  
  const [showTerminalMenu, setShowTerminalMenu] = useState(false);
  const [terminalMenuPos, setTerminalMenuPos] = useState({ x: 0, y: 0 });
  const terminalMenuRef = useRef<HTMLDivElement>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);

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
      setBluetooth(await invoke<BluetoothStatus>("get_bluetooth_status"));
      setMicMuted(await invoke<boolean>("get_mic_status"));
      setPrivacyMode(await invoke<boolean>("get_privacy_status"));
      setObsStatus(await invoke<ObsStatus>("get_obs_status"));
    } catch (e) {}
  };

  const fetchWeather = async (settings?: AppSettings) => {
    try {
      const s = settings || await invoke<AppSettings>("get_settings");
      if (s.weather_lat && s.weather_lon) {
        console.log("Refreshing weather for:", s.weather_location);
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
      setSettings(s);
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
      const s = event.payload;
      setSettings(s);
      setUse24h(s.use_24h !== false);
      if (s.accent_color) {
        document.documentElement.style.setProperty("--accent", s.accent_color);
      }
      // Force weather and media refresh on settings change
      fetchWeather(s);
      fetchMedia();
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
      if (terminalMenuRef.current && !terminalMenuRef.current.contains(event.target as Node)) {
        setShowTerminalMenu(false);
      }
      if (bluetoothRef.current && !bluetoothRef.current.contains(event.target as Node)) {
        setShowBluetoothMenu(false);
      }
    };

    if (showVolume || showPowerMenu || showWeather || showTerminalMenu || showBluetoothMenu) {
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
    if (showVolume || showPowerMenu || showWeather || showTerminalMenu || showBluetoothMenu) {
       invoke("set_window_height", { height: 760 }).catch(console.error);
    } else {
       invoke("set_window_height", { height: 32 }).catch(console.error);
    }
  }, [showVolume, showPowerMenu, showWeather, showTerminalMenu, showBluetoothMenu]);

  const handleVolumeChange = (newVol: number) => {
    setVolume(newVol);
    invoke("set_volume", { volume: newVol });
  };


  const handleMicToggle = async () => {
    try {
      const isMuted = await invoke<boolean>("toggle_mic_mute");
      setMicMuted(isMuted);
    } catch (e) {
      console.error("Mic toggle failed", e);
    }
  };

  const handlePrivacyToggle = async () => {
    try {
      const nextState = !privacyMode;
      await invoke("set_privacy_mode", { enabled: nextState });
      setPrivacyMode(nextState);
      // Privacy mode also affects mic state in our backend
      setMicMuted(nextState ? true : await invoke<boolean>("get_mic_status"));
    } catch (e) {
      console.error("Privacy mode toggle failed", e);
    }
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
                <div 
                  style={{ position: 'absolute' }} 
                  onClick={(e) => e.stopPropagation()}
                >
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

          <div 
            className={`status-item ${privacyMode ? 'privacy-active' : ''}`} 
            onClick={handlePrivacyToggle}
            title={privacyMode ? "Privacy Mode: ON (Mic/Cam blocked)" : "Privacy Mode: OFF"}
          >
            {privacyMode ? <Shield size={16} color="#ef4444" /> : <Shield size={16} />}
          </div>

          <div 
            className={`status-item ${bluetooth.connected ? 'bluetooth-active' : ''}`} 
            title="Bluetooth"
            ref={bluetoothRef}
            onClick={() => {
                setShowBluetoothMenu(!showBluetoothMenu);
                setShowVolume(false);
                setShowPowerMenu(false);
                setShowTerminalMenu(false);
            }}
          >
            <Bluetooth size={16} />
            {showBluetoothMenu && (
              <div className="bluetooth-popover dropdown" onClick={(e) => e.stopPropagation()}>
                <div className="ctx-header">Connected Devices</div>
                {bluetooth.devices.length > 0 ? (
                  bluetooth.devices.map((device, idx) => (
                    <div key={idx} className="device-item">
                      <Bluetooth size={12} style={{ marginRight: '8px', color: 'var(--accent)' }} />
                      {device}
                    </div>
                  ))
                ) : (
                  <div className="setting-hint" style={{ padding: '8px 12px' }}>No devices connected</div>
                )}
              </div>
            )}
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
              setShowTerminalMenu(false);
              setShowBluetoothMenu(false);
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

          <div 
            className="status-item" 
            onClick={() => {
                invoke("toggle_terminal_panel");
                setShowTerminalMenu(false);
            }} 
            onContextMenu={(e) => { 
                e.preventDefault(); 
                setTerminalMenuPos({ x: e.clientX, y: e.clientY });
                setShowTerminalMenu(true);
            }}
            ref={terminalMenuRef}
          >
            <TerminalIcon size={16} />
            {showTerminalMenu && (
                <div 
                    className="ctx-menu" 
                    style={{ 
                        position: 'fixed', 
                        top: '40px', 
                        right: 'auto',
                        left: `${terminalMenuPos.x - 100}px` 
                    }}
                    onClick={(e) => e.stopPropagation()}
                >
                    <div className="ctx-header">SSH Shortcuts</div>
                    {settings?.terminal_shortcuts.map((s, idx) => (
                        <button 
                            key={idx} 
                            onClick={async () => {
                                setShowTerminalMenu(false);
                                // Logic to start session (copied from backend command implementation)
                                const args = s.cmd.split(/\s+/);
                                await invoke("toggle_terminal_panel"); // Ensure open
                                await invoke("start_pty", { rows: 24, cols: 80, args }); // Start session
                            }}
                        >
                            <span>{s.label}</span>
                            <span className="ctx-hint">{s.cmd}</span>
                        </button>
                    ))}
                    {(!settings?.terminal_shortcuts || settings.terminal_shortcuts.length === 0) && (
                        <div style={{ padding: '8px 12px', fontSize: '12px', opacity: 0.5 }}>No shortcuts configured</div>
                    )}
                 </div>
            )}
          </div>

          <div 
            className="status-item" 
            ref={powerMenuRef}
            onClick={() => {
              setShowPowerMenu(!showPowerMenu);
              setShowVolume(false);
              setShowTerminalMenu(false);
              setShowBluetoothMenu(false);
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
