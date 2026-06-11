import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WeatherPopover } from "./WeatherPopover";
import { UsageLimitsPopover } from "./UsageLimitsPopover";
import { useMenuBarModel } from "./hooks/useMenuBarModel";
import { ProjectsPopover } from "./ProjectsPopover";
import { MowerPopover } from "./MowerPopover";
import { CameraPopover } from "./CameraPopover";
import { VacuumPopover } from "./VacuumPopover";
import { PhonePopover } from "./PhonePopover";
import { CalendarPopover } from "./CalendarPopover";
import {
  BarGroup, BarDivider, BarChip, BarItem, TrayIcon,
  MowerGlyph, SourceTag, Panel, KV, Mono, PBar,
} from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import type { LimitProvider } from "./contracts";

// The bar chip tracks the 5-hour window; the weekly window only shows in the popover.
function providerPct(p: LimitProvider): number {
  const v = p.shortWindow.remainingPercent ?? p.longWindow.remainingPercent;
  return v == null ? 100 : Math.round(v);
}

function chipTag(key: string): string {
  if (key.toLowerCase().includes("codex")) return "CDX";
  if (key.toLowerCase().includes("claude")) return "CLD";
  return key.slice(0, 3).toUpperCase();
}

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

  // Click-away listener
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (demoMode) return;
      const checks: Array<[{ current: HTMLElement | null }, () => void]> = [
        [volumeRef,       () => setShowVolume(false)],
        [powerMenuRef,    () => setShowPowerMenu(false)],
        [weatherRef,      () => setShowWeather(false)],
        [usageLimitsRef,  () => setShowUsageLimits(false)],
        [projectsRef,     () => setShowProjects(false)],
        [terminalMenuRef, () => setShowTerminalMenu(false)],
        [bluetoothRef,    () => setShowBluetoothMenu(false)],
        [mowerRef,        () => setShowMower(false)],
        [cameraRef,       () => setShowCamera(false)],
        [vacuumRef,       () => setShowVacuum(false)],
        [phoneRef,        () => setShowPhone(false)],
        [calendarRef,     () => setShowCalendar(false)],
      ];
      for (const [ref, close] of checks) {
        if (ref.current && !ref.current.contains(event.target as Node)) close();
      }
    };

    const anyOpen = showVolume || showPowerMenu || showWeather || showUsageLimits ||
      showProjects || showTerminalMenu || showBluetoothMenu || showMower ||
      showCamera || showVacuum || showPhone || showCalendar;

    if (anyOpen) document.addEventListener("mousedown", handleClickOutside);
    else document.removeEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [demoMode, showBluetoothMenu, showCalendar, showCamera, showMower, showPhone,
      showVacuum, showPowerMenu, showProjects, showTerminalMenu, showUsageLimits,
      showVolume, showWeather]);

  useEffect(() => {
    const anyOpen = showVolume || showPowerMenu || showWeather || showUsageLimits ||
      showProjects || showTerminalMenu || showBluetoothMenu || showMower ||
      showCamera || showVacuum || showPhone || showCalendar;
    invoke("set_window_height", { height: anyOpen ? 760 : 40 }).catch(console.error);
  }, [showVolume, showPowerMenu, showWeather, showUsageLimits, showProjects,
      showTerminalMenu, showBluetoothMenu, showMower, showCamera, showVacuum,
      showPhone, showCalendar]);

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
    changeVolume(newVol).catch((error) => console.error("Volume change failed", error));
  };

  const handleMicToggle = async () => {
    try { await toggleMic(); } catch (e) { console.error("Mic toggle failed", e); }
  };

  const handlePrivacyToggle = async () => {
    try { await togglePrivacy(); } catch (e) { console.error("Privacy mode toggle failed", e); }
  };

  const mediaEnabled = settings?.modules.media.enabled !== false;
  const camera = settings?.modules.camera;
  const cameraConfigured = !!(
    settings?.homeassistant_url && camera?.enabled && camera.entity_id
  );
  const cameraLabel = camera?.entity_id
    ? camera.entity_id.split(".")[1].replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase())
    : "Camera";
  const hasHome = !!(cameraConfigured || mower || vacuum);
  const hasPersonal = !!(phone || calendar !== null);

  const volPct = Math.round(volume * 100);

  return (
    <div className="main-window">
      <div className="menu-bar" onContextMenu={(e) => e.preventDefault()}>

        {/* ── DEV TOOLS ──────────────────────────────────────────────── */}
        <div className="left-section">
          <BarGroup gap={7}>

            {usageLimits && (() => {
              const hidden = settings?.usage_hidden_providers ?? [];
              const entries = Object.entries(usageLimits.providers)
                .filter(([key, p]) => p.enabled && !hidden.includes(key));
              if (entries.length === 0) return null;
              return (
                <div ref={usageLimitsRef} className="bar-anchor" style={{ gap: 5 }}>
                  {entries.map(([key, p]) => (
                    <BarChip
                      key={key}
                      tag={chipTag(key)}
                      pct={providerPct(p)}
                      onClick={() => setShowUsageLimits(!showUsageLimits)}
                    />
                  ))}
                  {showUsageLimits && <UsageLimitsPopover snapshot={usageLimits} />}
                </div>
              );
            })()}

            {usageLimits && projects && <BarDivider />}

            {projects && (
              <div ref={projectsRef} className="bar-anchor" style={{ gap: 5 }}>
                <BarItem
                  icon={<Icon name="branch" size={12} />}
                  hue="var(--accent)"
                  mono={String(projects.projects.length)}
                  onClick={() => setShowProjects(!showProjects)}
                />
                {projects.attentionCount > 0 && (
                  <span style={{ padding: '1.5px 6px', borderRadius: 4, background: `color-mix(in srgb, ${HUE.red} 18%, transparent)`, border: `1px solid color-mix(in srgb, ${HUE.red} 35%, transparent)` }}>
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600, color: HUE.red }}>{projects.attentionCount}</span>
                  </span>
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

          </BarGroup>
        </div>

        {/* ── NOW PLAYING ────────────────────────────────────────────── */}
        <div className="center-section">
          {!mediaEnabled ? null : mediaInfo ? (
            <BarGroup gap={4}>
              <span style={{ display: 'inline-flex', gap: 2, marginRight: 4 }}>
                <span style={{ color: T.t3, display: 'flex', padding: 3, cursor: 'pointer' }}
                  onClick={(e) => { e.stopPropagation(); handleMediaControl("previous"); }}>
                  <Icon name="prev" size={10} />
                </span>
                <span style={{ color: T.t2, display: 'flex', padding: 3, cursor: 'pointer' }}
                  onClick={(e) => { e.stopPropagation(); handleMediaControl("play_pause"); }}>
                  {mediaInfo.is_playing ? <Icon name="pause" size={10} /> : <Icon name="play" size={10} />}
                </span>
                <span style={{ color: T.t3, display: 'flex', padding: 3, cursor: 'pointer' }}
                  onClick={(e) => { e.stopPropagation(); handleMediaControl("next"); }}>
                  <Icon name="next" size={10} />
                </span>
              </span>
              <BarItem icon={<Icon name="music" size={11} />} hue={HUE.media} text={mediaInfo.artist}
                onClick={() => invoke("toggle_expanded_player")} />
              <span style={{ fontSize: 11.5, fontWeight: 600, color: T.t1, whiteSpace: 'nowrap', maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis' }}
                onClick={() => invoke("toggle_expanded_player")}>
                {mediaInfo.title}
              </span>
              <span style={{ marginLeft: 4 }}>
                <SourceTag name={mediaInfo.source === 'plex' ? 'PLEX' : 'SYSTEM'} />
              </span>
            </BarGroup>
          ) : (
            <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, color: T.t3, fontSize: 11.5 }}>
              <Icon name="music" size={11} />
              Nothing Playing
            </span>
          )}
        </div>

        {/* ── RIGHT CLUSTERS ─────────────────────────────────────────── */}
        <div className="right-section">

          {/* ENVIRONMENT */}
          {weather && (
            <div ref={weatherRef} className="bar-anchor">
              <BarGroup>
                <BarItem
                  icon={<Icon name="cloud" size={12} />}
                  hue={HUE.weather}
                  mono={`${Math.round(weather.temp)}°`}
                  onClick={() => setShowWeather(!showWeather)}
                />
              </BarGroup>
              {showWeather && <WeatherPopover data={weather} onClose={() => setShowWeather(false)} />}
            </div>
          )}

          {/* HOME */}
          {hasHome && <BarDivider />}
          {hasHome && (
            <BarGroup gap={10}>

              {cameraConfigured && (
                <div ref={cameraRef} className="bar-anchor">
                  <BarItem
                    icon={<Icon name="cam" size={12} />}
                    hue={T.t3}
                    text={cameraLabel}
                    onClick={() => setShowCamera(!showCamera)}
                  />
                  {showCamera && <CameraPopover label={cameraLabel} />}
                </div>
              )}

              {mower && (() => {
                const gc = mower.state === 'mowing' ? HUE.ok
                  : mower.state === 'error' ? HUE.red
                  : HUE.mower;
                return (
                  <div ref={mowerRef} className="bar-anchor">
                    <BarItem
                      icon={<MowerGlyph size={13} color={gc} />}
                      text={mower.state_label}
                      onClick={() => setShowMower(!showMower)}
                    />
                    {showMower && <MowerPopover mower={mower} />}
                  </div>
                );
              })()}

              {vacuum && (() => {
                const vc = vacuum.cleaning ? HUE.ok
                  : vacuum.status === 'charging' ? HUE.amber
                  : HUE.vacuum;
                const vt = vacuum.cleaning
                  ? `${vacuum.cleaning_progress}%`
                  : vacuum.status.charAt(0).toUpperCase() + vacuum.status.slice(1);
                return (
                  <div ref={vacuumRef} className="bar-anchor">
                    <BarItem
                      icon={<Icon name={vacuum.charging ? "bolt" : "battery"} size={12} />}
                      hue={vc}
                      text={vt}
                      onClick={() => setShowVacuum(!showVacuum)}
                    />
                    {showVacuum && <VacuumPopover vacuum={vacuum} />}
                  </div>
                );
              })()}

            </BarGroup>
          )}

          {/* PERSONAL */}
          {hasPersonal && <BarDivider />}
          {hasPersonal && (
            <BarGroup gap={10}>

              {phone && (() => {
                return (
                  <div ref={phoneRef} className="bar-anchor">
                    <BarItem
                      icon={<Icon name={phone.charging ? "bolt" : "battery"} size={12} />}
                      hue={HUE.phone}
                      mono={`${phone.battery}%`}
                      onClick={() => setShowPhone(!showPhone)}
                    />
                    {showPhone && <PhonePopover phone={phone} />}
                  </div>
                );
              })()}

              {calendar !== null && (() => {
                const now = new Date();
                const next = calendar.find(e => !e.all_day && new Date(e.end) > now);
                const ongoing = next && new Date(next.start) <= now;
                const calHue = ongoing ? 'var(--accent)' : next ? HUE.cal : undefined;
                const calTime = next
                  ? new Date(next.start).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: false })
                  : undefined;
                const calText = next ? `${calTime} ${next.summary}` : undefined;
                return (
                  <div ref={calendarRef} className="bar-anchor">
                    <BarItem
                      icon={<Icon name="cal" size={11} />}
                      hue={calHue}
                      text={calText}
                      dim={!next}
                      style={{ maxWidth: 200 }}
                      onClick={() => setShowCalendar(!showCalendar)}
                    />
                    {showCalendar && <CalendarPopover events={calendar} />}
                  </div>
                );
              })()}

            </BarGroup>
          )}

          {/* SYSTEM */}
          <BarDivider />
          <BarGroup gap={2}>

            {obsStatus && (obsStatus.is_recording || obsStatus.is_streaming) && (
              <TrayIcon icon={<Icon name="cam" size={11} />} state="alert" />
            )}

            <TrayIcon
              icon={<Icon name="shield" size={11} />}
              state={privacyMode ? 'active' : 'idle'}
              onClick={handlePrivacyToggle}
            />

            <div ref={bluetoothRef} className="bar-anchor">
              <TrayIcon
                icon={<Icon name="bt" size={11} />}
                state={showBluetoothMenu ? 'open' : bluetooth.connected ? 'active' : 'idle'}
                onClick={() => {
                  setShowBluetoothMenu(!showBluetoothMenu);
                  setShowVolume(false);
                  setShowPowerMenu(false);
                  setShowTerminalMenu(false);
                }}
              />
              {showBluetoothMenu && (
                <Panel w={200} title="Connected Devices" hue="var(--accent)" style={{ right: 0 }}>
                  {bluetooth.devices.length > 0
                    ? bluetooth.devices.map((device, idx) => (
                        <KV key={idx} icon={<Icon name="bt" size={11} />} label={device} hue="var(--accent)" />
                      ))
                    : <div style={{ fontSize: 12, color: T.t3 }}>No devices connected</div>
                  }
                </Panel>
              )}
            </div>

            <TrayIcon
              icon={<Icon name="mic" size={11} />}
              state={micMuted ? 'alert' : 'idle'}
              onClick={handleMicToggle}
            />

            {/* Volume */}
            <div ref={volumeRef} className="bar-anchor">
              <TrayIcon
                icon={<Icon name="vol" size={11} />}
                state={showVolume ? 'open' : 'idle'}
                onClick={() => {
                  setShowVolume(!showVolume);
                  setShowPowerMenu(false);
                  setShowTerminalMenu(false);
                  setShowBluetoothMenu(false);
                }}
              />
              {showVolume && (
                <Panel w={300} title="Volume" style={{ right: 0 }}>
                  {/* Slider row */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '4px 2px 2px' }}>
                    <span style={{ color: T.t2, display: 'flex', cursor: 'pointer' }}><Icon name="vol" size={14} /></span>
                    <div style={{ flex: 1, position: 'relative', height: 16, display: 'flex', alignItems: 'center' }}>
                      <PBar pct={volPct} hue="var(--accent)" h={4} />
                      <input
                        type="range" min="0" max="1" step="0.01" value={volume}
                        onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
                        autoFocus
                        style={{ position: 'absolute', inset: 0, opacity: 0, cursor: 'grab', width: '100%', margin: 0 }}
                      />
                      <span style={{ position: 'absolute', left: `${volPct}%`, transform: 'translateX(-50%)', width: 13, height: 13, borderRadius: 999, background: T.t1, boxShadow: '0 2px 6px rgba(0,0,0,0.5)', pointerEvents: 'none' }} />
                    </div>
                    <Mono size={11.5} w={600} style={{ width: 24, textAlign: 'right' }}>{volPct}</Mono>
                  </div>
                </Panel>
              )}
            </div>

            {battery?.has_battery && (
              <BarItem icon={<Icon name="battery" size={11} />} hue={T.t3} mono={`${battery.percentage}%`} />
            )}

            {/* Terminal SSH shortcuts */}
            <div
              ref={terminalMenuRef}
              className="bar-anchor"
              onContextMenu={(e) => {
                e.preventDefault();
                setTerminalMenuPos({ x: e.clientX, y: e.clientY });
                setShowTerminalMenu(true);
              }}
            >
              <TrayIcon
                icon={<Icon name="term" size={11} />}
                state={showTerminalMenu ? 'open' : 'idle'}
                onClick={() => {
                  invoke("toggle_terminal_panel");
                  setShowTerminalMenu(false);
                }}
              />
              {showTerminalMenu && (
                <Panel
                  w={220}
                  title="SSH Shortcuts"
                  style={{ position: 'fixed', top: 48, left: terminalMenuPos.x - 110 }}
                  onClick={(e) => e.stopPropagation()}
                >
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                    {settings?.terminal_shortcuts.map((s, idx) => (
                      <div
                        key={idx}
                        onClick={async () => {
                          setShowTerminalMenu(false);
                          await invoke("toggle_terminal_panel");
                          await invoke("start_pty", { rows: 24, cols: 80, command: s.cmd });
                        }}
                        style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: 8, padding: '6px 0', cursor: 'pointer', borderBottom: idx < (settings?.terminal_shortcuts.length ?? 0) - 1 ? `1px solid ${T.divider}` : 'none' }}
                      >
                        <span style={{ fontSize: 12, color: T.t2 }}>{s.label}</span>
                        <Mono size={10} color={T.t3}>{s.cmd}</Mono>
                      </div>
                    ))}
                    {(!settings?.terminal_shortcuts || settings.terminal_shortcuts.length === 0) && (
                      <div style={{ fontSize: 12, color: T.t3 }}>No shortcuts configured</div>
                    )}
                  </div>
                </Panel>
              )}
            </div>

            {/* Power menu — Panel with SYSTEM header + icon rows */}
            <div ref={powerMenuRef} className="bar-anchor">
              <TrayIcon
                icon={<Icon name="power" size={11} />}
                state={showPowerMenu ? 'open' : 'idle'}
                onClick={() => {
                  setShowPowerMenu(!showPowerMenu);
                  setShowVolume(false);
                  setShowTerminalMenu(false);
                  setShowBluetoothMenu(false);
                }}
              />
              {showPowerMenu && (
                <Panel w={172} pad={8} title="System" style={{ right: 0 }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                    {([
                      ['Lock',      'lock',     'lock',    false],
                      ['Sleep',     'sleep',    'moon',    false],
                      ['Restart',   'restart',  'restart', false],
                      ['Shut Down', 'shutdown', 'power',   true ],
                    ] as const).map(([label, action, iconName, isDanger]) => (
                      <div
                        key={action}
                        onClick={() => invoke("system_power_action", { action })}
                        style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '8px 10px', borderRadius: T.ctlR, cursor: 'pointer', color: isDanger ? HUE.red : T.t2, fontSize: 12.5 }}
                        onMouseEnter={(e) => { e.currentTarget.style.background = T.ctlBg; }}
                        onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; }}
                      >
                        <span style={{ display: 'flex', color: isDanger ? HUE.red : T.t3 }}>
                          <Icon name={iconName} size={12} />
                        </span>
                        {label}
                      </div>
                    ))}
                  </div>
                </Panel>
              )}
            </div>

            <TrayIcon
              icon={<Icon name="gear" size={11} />}
              state="idle"
              onClick={() => invoke("open_settings").catch((e) => console.error("open_settings failed:", e))}
            />

          </BarGroup>

          {/* CLOCK */}
          <BarDivider />
          <BarGroup>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, fontWeight: 600, color: T.t1, whiteSpace: 'nowrap' }}>
              {time}
            </span>
          </BarGroup>

        </div>
      </div>
    </div>
  );
}

export default App;
