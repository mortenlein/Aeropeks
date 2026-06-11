import { Fragment, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useMenuBarModel } from "./hooks/useMenuBarModel";
import { BAR_MODULES, type BarModuleDef, type BarSection } from "./modules";
import {
  BarGroup, BarDivider, BarItem, TrayIcon,
  SourceTag, Panel, KV, Mono, PBar,
} from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";

// Popovers forced open while screenshot mode poses the bar.
const DEMO_OPEN_POPOVERS = ["volume", "power", "bluetooth"];

function App() {
  const model = useMenuBarModel();
  const {
    battery,
    bluetooth,
    changeVolume,
    controlMedia,
    mediaInfo,
    micMuted,
    obsStatus,
    privacyMode,
    settings,
    time,
    toggleMic,
    togglePrivacy,
    volume,
  } = model;

  const [openPopover, setOpenPopover] = useState<string | null>(null);
  const [demoMode, setDemoMode] = useState(false);
  const [terminalMenuPos, setTerminalMenuPos] = useState({ x: 0, y: 0 });

  const togglePopover = (id: string) =>
    setOpenPopover((current) => (current === id ? null : id));
  const isOpen = (id: string) =>
    openPopover === id || (demoMode && DEMO_OPEN_POPOVERS.includes(id));

  // One click-away handler: anything outside the open anchor closes it.
  useEffect(() => {
    if (openPopover === null || demoMode) return;
    const handleMouseDown = (event: MouseEvent) => {
      const anchor = (event.target as HTMLElement).closest?.("[data-popover-id]");
      if (!(anchor instanceof HTMLElement) || anchor.dataset.popoverId !== openPopover) {
        setOpenPopover(null);
      }
    };
    document.addEventListener("mousedown", handleMouseDown);
    return () => document.removeEventListener("mousedown", handleMouseDown);
  }, [openPopover, demoMode]);

  // The bar window grows while any popover needs room below it.
  const anyOpen = openPopover !== null || demoMode;
  useEffect(() => {
    invoke("set_window_height", { height: anyOpen ? 760 : 40 }).catch(console.error);
  }, [anyOpen]);

  useEffect(() => {
    const unlistenEnter = listen("demo-mode", () => setDemoMode(true));
    const unlistenExit = listen("demo-mode-exit", () => {
      setDemoMode(false);
      setOpenPopover(null);
    });
    return () => {
      unlistenEnter.then((f) => f());
      unlistenExit.then((f) => f());
    };
  }, []);

  const handleMediaControl = async (action: string) => {
    try {
      await controlMedia(action as "previous" | "play_pause" | "next");
    } catch (e) {
      console.error("Media control failed", e);
    }
  };

  const handleVolumeChange = (newVol: number) => {
    changeVolume(newVol).catch((error) => console.error("Volume change failed", error));
  };

  const handleMicToggle = async () => {
    try { await toggleMic(); } catch (e) { console.error("Mic toggle failed", e); }
  };

  const handlePrivacyToggle = async () => {
    try { await togglePrivacy(); } catch (e) { console.error("Privacy mode toggle failed", e); }
  };

  const renderModule = (def: BarModuleDef) => {
    const ctx = { open: isOpen(def.id), toggle: () => togglePopover(def.id) };
    return (
      <div
        key={def.id}
        className="bar-anchor"
        data-popover-id={def.id}
        style={def.anchorStyle}
      >
        {def.item(model, ctx)}
        {ctx.open && def.popover(model, ctx)}
      </div>
    );
  };

  const sectionModules = (section: BarSection) =>
    BAR_MODULES.filter((def) => def.section === section && def.visible(model));

  const devModules = sectionModules("dev");
  const environmentModules = sectionModules("environment");
  const homeModules = sectionModules("home");
  const personalModules = sectionModules("personal");

  const mediaEnabled = settings?.modules.media.enabled !== false;
  const volPct = Math.round(volume * 100);

  return (
    <div className="main-window">
      <div className="menu-bar" onContextMenu={(e) => e.preventDefault()}>

        {/* ── DEV TOOLS ──────────────────────────────────────────────── */}
        <div className="left-section">
          <BarGroup gap={7}>
            {devModules.map((def, index) => (
              <Fragment key={def.id}>
                {index > 0 && <BarDivider />}
                {renderModule(def)}
              </Fragment>
            ))}
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
          {environmentModules.length > 0 && (
            <BarGroup>{environmentModules.map(renderModule)}</BarGroup>
          )}

          {/* HOME */}
          {homeModules.length > 0 && (
            <>
              <BarDivider />
              <BarGroup gap={10}>{homeModules.map(renderModule)}</BarGroup>
            </>
          )}

          {/* PERSONAL */}
          {personalModules.length > 0 && (
            <>
              <BarDivider />
              <BarGroup gap={10}>{personalModules.map(renderModule)}</BarGroup>
            </>
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

            <div className="bar-anchor" data-popover-id="bluetooth">
              <TrayIcon
                icon={<Icon name="bt" size={11} />}
                state={isOpen("bluetooth") ? 'open' : bluetooth.connected ? 'active' : 'idle'}
                onClick={() => togglePopover("bluetooth")}
              />
              {isOpen("bluetooth") && (
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
            <div className="bar-anchor" data-popover-id="volume">
              <TrayIcon
                icon={<Icon name="vol" size={11} />}
                state={isOpen("volume") ? 'open' : 'idle'}
                onClick={() => togglePopover("volume")}
              />
              {isOpen("volume") && (
                <Panel w={300} title="Volume" style={{ right: 0 }}>
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
              className="bar-anchor"
              data-popover-id="terminal"
              onContextMenu={(e) => {
                e.preventDefault();
                setTerminalMenuPos({ x: e.clientX, y: e.clientY });
                setOpenPopover("terminal");
              }}
            >
              <TrayIcon
                icon={<Icon name="term" size={11} />}
                state={isOpen("terminal") ? 'open' : 'idle'}
                onClick={() => {
                  invoke("toggle_terminal_panel");
                  setOpenPopover(null);
                }}
              />
              {isOpen("terminal") && (
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
                          setOpenPopover(null);
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

            {/* Power menu */}
            <div className="bar-anchor" data-popover-id="power">
              <TrayIcon
                icon={<Icon name="power" size={11} />}
                state={isOpen("power") ? 'open' : 'idle'}
                onClick={() => togglePopover("power")}
              />
              {isOpen("power") && (
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
