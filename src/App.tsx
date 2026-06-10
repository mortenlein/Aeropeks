import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Music, Volume2, Clock, Play, Pause, SkipBack, SkipForward, Terminal as TerminalIcon, Bluetooth, Battery, Mic, MicOff, Video, Cloud, Power, Settings as SettingsIcon, Shield, FolderGit2, Camera, Bot, Smartphone, CalendarDays } from "lucide-react";
import dreameIcon from "./assets/dreame_icon.png";
import { WeatherPopover } from "./WeatherPopover";
import {
  lowestRemaining,
  UsageLimitsPopover,
  UsageLimitsSummary,
} from "./UsageLimitsPopover";
import { useMenuBarModel } from "./hooks/useMenuBarModel";
import { ProjectsPopover } from "./ProjectsPopover";
import { MowerPopover } from "./MowerPopover";
import { CameraPopover } from "./CameraPopover";
import { VacuumPopover } from "./VacuumPopover";
import { PhonePopover } from "./PhonePopover";
import { CalendarPopover } from "./CalendarPopover";

function App() {
  const {
    battery,
    bluetooth,
    changeVolume,
    controlMedia,
    mediaInfo,
    micMuted,
    obsStatus,
    privacyMode,
    projects,
    projectsRefreshing,
    refreshProjects,
    settings,
    time,
    toggleMic,
    togglePrivacy,
    calendar,
    mower,
    phone,
    vacuum,
    usageLimits,
    volume,
    weather,
  } = useMenuBarModel();
  const [showVolume, setShowVolume] = useState(false);
  const [showBluetoothMenu, setShowBluetoothMenu] = useState(false);
  const bluetoothRef = useRef<HTMLDivElement>(null);
  const [showWeather, setShowWeather] = useState(false);
  const weatherRef = useRef<HTMLDivElement>(null);
  const [showUsageLimits, setShowUsageLimits] = useState(false);
  const usageLimitsRef = useRef<HTMLDivElement>(null);
  const [showProjects, setShowProjects] = useState(false);
  const projectsRef = useRef<HTMLDivElement>(null);
  const [showMower, setShowMower] = useState(false);
  const mowerRef = useRef<HTMLDivElement>(null);
  const [showCamera, setShowCamera] = useState(false);
  const cameraRef = useRef<HTMLDivElement>(null);
  const [showVacuum, setShowVacuum] = useState(false);
  const vacuumRef = useRef<HTMLDivElement>(null);
  const [showPhone, setShowPhone] = useState(false);
  const phoneRef = useRef<HTMLDivElement>(null);
  const [showCalendar, setShowCalendar] = useState(false);
  const calendarRef = useRef<HTMLDivElement>(null);
  
  const [showPowerMenu, setShowPowerMenu] = useState(false);
  const powerMenuRef = useRef<HTMLDivElement>(null);
  
  const [showTerminalMenu, setShowTerminalMenu] = useState(false);
  const [terminalMenuPos, setTerminalMenuPos] = useState({ x: 0, y: 0 });
  const terminalMenuRef = useRef<HTMLDivElement>(null);
  const volumeRef = useRef<HTMLDivElement>(null);
  const [demoMode, setDemoMode] = useState(false);

  const handleMediaControl = async (action: string) => {
    try {
      await controlMedia(action as "previous" | "play_pause" | "next");
    } catch (e) {
      console.error("Media control failed", e);
    }
  };

  // Click-away listener — suspended in demo/screenshot mode
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (demoMode) return;
      if (volumeRef.current && !volumeRef.current.contains(event.target as Node)) {
        setShowVolume(false);
      }
      if (powerMenuRef.current && !powerMenuRef.current.contains(event.target as Node)) {
        setShowPowerMenu(false);
      }
      if (weatherRef.current && !weatherRef.current.contains(event.target as Node)) {
        setShowWeather(false);
      }
      if (usageLimitsRef.current && !usageLimitsRef.current.contains(event.target as Node)) {
        setShowUsageLimits(false);
      }
      if (projectsRef.current && !projectsRef.current.contains(event.target as Node)) {
        setShowProjects(false);
      }
      if (terminalMenuRef.current && !terminalMenuRef.current.contains(event.target as Node)) {
        setShowTerminalMenu(false);
      }
      if (bluetoothRef.current && !bluetoothRef.current.contains(event.target as Node)) {
        setShowBluetoothMenu(false);
      }
      if (mowerRef.current && !mowerRef.current.contains(event.target as Node)) {
        setShowMower(false);
      }
      if (cameraRef.current && !cameraRef.current.contains(event.target as Node)) {
        setShowCamera(false);
      }
      if (vacuumRef.current && !vacuumRef.current.contains(event.target as Node)) {
        setShowVacuum(false);
      }
      if (phoneRef.current && !phoneRef.current.contains(event.target as Node)) {
        setShowPhone(false);
      }
      if (calendarRef.current && !calendarRef.current.contains(event.target as Node)) {
        setShowCalendar(false);
      }
    };

    if (showVolume || showPowerMenu || showWeather || showUsageLimits || showProjects || showTerminalMenu || showBluetoothMenu || showMower || showCamera || showVacuum || showPhone || showCalendar) {
      document.addEventListener("mousedown", handleClickOutside);
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [
    demoMode,
    showBluetoothMenu,
    showCalendar,
    showCamera,
    showMower,
    showPhone,
    showVacuum,
    showPowerMenu,
    showProjects,
    showTerminalMenu,
    showUsageLimits,
    showVolume,
    showWeather,
  ]);

  useEffect(() => {
    // Dynamic window expansion for dropdowns/popovers
    if (showVolume || showPowerMenu || showWeather || showUsageLimits || showProjects || showTerminalMenu || showBluetoothMenu || showMower || showCamera || showVacuum || showPhone || showCalendar) {
       invoke("set_window_height", { height: 760 }).catch(console.error);
    } else {
       invoke("set_window_height", { height: 32 }).catch(console.error);
    }
  }, [showVolume, showPowerMenu, showWeather, showUsageLimits, showProjects, showTerminalMenu, showBluetoothMenu, showMower, showCamera, showVacuum, showPhone, showCalendar]);

  useEffect(() => {
    const unlistenEnter = listen("demo-mode", () => {
      setDemoMode(true);
      setShowPowerMenu(true);
      setShowVolume(true);
      setShowBluetoothMenu(true);
    });
    const unlistenExit = listen("demo-mode-exit", () => {
      setDemoMode(false);
      setShowPowerMenu(false);
      setShowVolume(false);
      setShowBluetoothMenu(false);
    });
    return () => {
      unlistenEnter.then((f) => f());
      unlistenExit.then((f) => f());
    };
  }, []);

  const handleVolumeChange = (newVol: number) => {
    changeVolume(newVol).catch((error) =>
      console.error("Volume change failed", error),
    );
  };


  const handleMicToggle = async () => {
    try {
      await toggleMic();
    } catch (e) {
      console.error("Mic toggle failed", e);
    }
  };

  const handlePrivacyToggle = async () => {
    try {
      await togglePrivacy();
    } catch (e) {
      console.error("Privacy mode toggle failed", e);
    }
  };
  return (
    <div className="main-window">
      <div className="menu-bar" onContextMenu={(e) => e.preventDefault()}>
        <div className="left-section">
          {usageLimits && lowestRemaining(usageLimits) !== null && (
            <div
              className={`status-item usage-limits-item ${
                (lowestRemaining(usageLimits) ?? 100) <= 20 ? "usage-critical" : ""
              }`}
              ref={usageLimitsRef}
              title="AI usage limits"
              onClick={() => setShowUsageLimits(!showUsageLimits)}
            >
              <UsageLimitsSummary snapshot={usageLimits} />
              {showUsageLimits && (
                <UsageLimitsPopover snapshot={usageLimits} />
              )}
            </div>
          )}

          {projects && (
            <div
              className="status-item projects-item"
              ref={projectsRef}
              title={`${projects.attentionCount} projects need attention`}
              onClick={() => setShowProjects(!showProjects)}
            >
              <FolderGit2 size={15} />
              <span
                className={
                  projects.averageHealth >= 80
                    ? "project-score-ok"
                    : projects.averageHealth >= 60
                      ? "project-score-warn"
                      : "project-score-bad"
                }
              >
                {projects.averageHealth}
              </span>
              {projects.attentionCount > 0 && (
                <small>{projects.attentionCount}</small>
              )}
              {showProjects && (
                <ProjectsPopover
                  snapshot={projects}
                  refreshing={projectsRefreshing}
                  onRefresh={() => refreshProjects().catch(console.error)}
                />
              )}
            </div>
          )}
        </div>
        
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

          {settings?.homeassistant_url && (
            <div
              ref={cameraRef}
              className="status-item"
              title="Garage Camera"
              onClick={() => setShowCamera(!showCamera)}
            >
              <Camera size={14} />
              <span>Garage</span>
              {showCamera && (
                <div style={{ position: 'absolute', inset: 0 }} onClick={(e) => e.stopPropagation()}>
                  <CameraPopover />
                </div>
              )}
            </div>
          )}

          {mower && (() => {
            const mowerColor = ({ mowing: '#4ade80', paused: '#fb923c', error: '#ef4444', docked: '#a78bfa' } as Record<string, string>)[mower.state] ?? 'rgba(255,255,255,0.35)';
            return (
              <div
                ref={mowerRef}
                className={`status-item${mower.state === 'mowing' ? ' mower-active' : ''}`}
                style={{ color: mowerColor }}
                title={`Mower: ${mower.state_label} · fw${mower.firmware}`}
                onClick={() => setShowMower(!showMower)}
              >
                <img src={dreameIcon} style={{ width: 16, height: 16, objectFit: 'contain', opacity: mower.state === 'error' ? 0.4 : 1 }} alt="" />
                <span>{mower.state_label}</span>
                {showMower && (
                  <div style={{ position: 'absolute', inset: 0 }} onClick={(e) => e.stopPropagation()}>
                    <MowerPopover mower={mower} />
                  </div>
                )}
              </div>
            );
          })()}

          {vacuum && (() => {
            const vacuumColor = ({ cleaning: '#4ade80', returning: '#60a5fa', paused: '#fb923c', error: '#ef4444', charging: '#fbbf24' } as Record<string, string>)[vacuum.status] ?? 'rgba(255,255,255,0.5)';
            return (
              <div
                ref={vacuumRef}
                className="status-item"
                style={{ color: vacuumColor }}
                title={`Roberto: ${vacuum.status}`}
                onClick={() => setShowVacuum(!showVacuum)}
              >
                <Bot size={14} />
                <span>{vacuum.cleaning ? `${vacuum.cleaning_progress}%` : vacuum.status.charAt(0).toUpperCase() + vacuum.status.slice(1)}</span>
                {showVacuum && (
                  <div style={{ position: 'absolute', inset: 0 }} onClick={(e) => e.stopPropagation()}>
                    <VacuumPopover vacuum={vacuum} />
                  </div>
                )}
              </div>
            );
          })()}

          {phone && (() => {
            const battColor = phone.charging ? '#4ade80' : phone.battery <= 20 ? '#ef4444' : phone.battery <= 40 ? '#fb923c' : 'rgba(255,255,255,0.5)';
            return (
              <div
                ref={phoneRef}
                className="status-item"
                style={{ color: battColor }}
                title={`Pixel 9 Pro XL · ${phone.battery}% · ${phone.at_home ? 'Home' : 'Away'}`}
                onClick={() => setShowPhone(!showPhone)}
              >
                <Smartphone size={13} />
                <span>{phone.battery}%</span>
                {showPhone && (
                  <div style={{ position: 'absolute', inset: 0 }} onClick={(e) => e.stopPropagation()}>
                    <PhonePopover phone={phone} />
                  </div>
                )}
              </div>
            );
          })()}

          {calendar !== null && (() => {
            const now = new Date();
            const next = calendar.find(e => !e.all_day && new Date(e.end) > now);
            const ongoing = next && new Date(next.start) <= now;
            const calColor = ongoing ? '#4ade80' : next ? '#818cf8' : 'rgba(255,255,255,0.35)';
            return (
              <div
                ref={calendarRef}
                className="status-item"
                style={{ color: calColor }}
                title="Google Calendar"
                onClick={() => setShowCalendar(!showCalendar)}
              >
                <CalendarDays size={13} />
                {next && !ongoing && (
                  <span style={{ fontSize: '11px', fontVariantNumeric: 'tabular-nums' }}>
                    {new Date(next.start).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: false })}
                  </span>
                )}
                {ongoing && next && (
                  <span style={{ fontSize: '11px', maxWidth: '200px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {next.summary}
                  </span>
                )}
                {showCalendar && (
                  <div style={{ position: 'absolute', inset: 0 }} onClick={(e) => e.stopPropagation()}>
                    <CalendarPopover events={calendar} />
                  </div>
                )}
              </div>
            );
          })()}

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
                                await invoke("toggle_terminal_panel"); // Ensure open
                                await invoke("start_pty", { rows: 24, cols: 80, command: s.cmd });
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

          <div className="status-item" onClick={() => invoke("open_settings").catch((e) => console.error("open_settings failed:", e))}>
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
