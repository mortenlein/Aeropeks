import { type CSSProperties, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsModel } from "./hooks/useSettingsModel";
import { T } from "./tokens";
import { Card, Micro } from "./atoms";

// ── Local settings atoms ───────────────────────────────────────────
const SWATCHES = ["#22C55E", "#38BDF8", "#A78BFA", "#F4845F"];

function SetSection({ title, action, children }: { title: string; action?: ReactNode; children: ReactNode }) {
  return (
    <div style={{ marginBottom: 16 }}>
      <Card pad={18} style={{ display: "flex", flexDirection: "column", gap: 16 }}>
        <div style={{ display: "flex", alignItems: "center" }}>
          <Micro color="var(--accent)">{title}</Micro>
          <span style={{ flex: 1 }} />
          {action}
        </div>
        {children}
      </Card>
    </div>
  );
}

function SetField({ label, help, children }: {
  label?: string;
  help?: string;
  children: ReactNode;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
      {label && (
        <span style={{ fontSize: 12, fontWeight: 600, color: T.t1 }}>{label}</span>
      )}
      <div style={{ display: "flex", gap: 8 }}>{children}</div>
      {help && (
        <span style={{ fontSize: 10.5, color: T.t3, lineHeight: 1.5 }}>{help}</span>
      )}
    </div>
  );
}

function Inp({ style, ...props }: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      style={{
        background: T.inputBg,
        border: T.inputBorder,
        borderRadius: T.ctlR,
        padding: "7px 10px",
        fontFamily: "var(--font-ui)",
        fontSize: 12,
        color: T.t1,
        outline: "none",
        flex: 1,
        minWidth: 0,
        boxSizing: "border-box",
        ...style,
      }}
    />
  );
}

function Btn({ primary, children, onClick, disabled, style }: {
  primary?: boolean;
  children: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  style?: CSSProperties;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: "8px 18px",
        borderRadius: T.ctlR,
        fontFamily: "var(--font-ui)",
        fontSize: 12,
        fontWeight: primary ? 700 : 500,
        cursor: disabled ? "not-allowed" : "pointer",
        background: primary ? "var(--accent)" : T.ctlBg,
        border: primary ? "none" : `1px solid ${T.divider}`,
        color: primary ? "#10131a" : T.t2,
        opacity: disabled ? 0.5 : 1,
        ...style,
      }}
    >
      {children}
    </button>
  );
}

function SetCheck({ on, onChange, label, help }: {
  on: boolean;
  onChange: (v: boolean) => void;
  label: string;
  help?: string;
}) {
  return (
    <div style={{ display: "flex", gap: 10, cursor: "pointer" }} onClick={() => onChange(!on)}>
      <span style={{
        width: 16, height: 16, borderRadius: Math.min(T.ctlR, 5), flexShrink: 0, marginTop: 1,
        background: on ? "var(--accent)" : T.inputBg,
        border: on ? "1px solid transparent" : T.inputBorder,
        display: "flex", alignItems: "center", justifyContent: "center", color: "#10131a",
      }}>
        {on && (
          <svg width="9" height="9" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            <path d="M1.5 5.5l2.5 2.5 4.5-5" />
          </svg>
        )}
      </span>
      <div>
        <div style={{ fontSize: 12.5, fontWeight: 600, color: T.t1 }}>{label}</div>
        {help && <div style={{ fontSize: 10.5, color: T.t3, marginTop: 3, lineHeight: 1.5 }}>{help}</div>}
      </div>
    </div>
  );
}

// ── Settings component ─────────────────────────────────────────────
function Settings() {
  const {
    shortcuts,
    addShortcut,
    removeShortcut,
    updateShortcut,
    searchQuery,
    handleSearch,
    isSearching,
    searchResults,
    selectLocation,
    weatherLocation,
    weatherLat,
    weatherLon,
    githubToken,
    setGithubToken,
    usageLimitsUrl,
    setUsageLimitsUrl,
    obsUrl,
    setObsUrl,
    obsPassword,
    setObsPassword,
    dreameUsername,
    setDreameUsername,
    dreamePassword,
    setDreamePassword,
    dreameDeviceId,
    setDreameDeviceId,
    haUrl,
    setHaUrl,
    haToken,
    setHaToken,
    haCalendarEntityId,
    setHaCalendarEntityId,
    reserveScreenSpace,
    setReserveScreenSpace,
    hideNativeTaskbar,
    setHideNativeTaskbar,
    handleRestoreShell,
    handleClearIconCache,
    shellMessage,
    debugInspector,
    setDebugInspector,
    refreshDebugWindows,
    debugWindows,
    accentColor,
    setAccentColor,
    use24h,
    setUse24h,
    saved,
    handleSave,
  } = useSettingsModel();

  return (
    <div style={{ fontFamily: "var(--font-ui)", background: T.panelBg, color: T.t1, height: "100vh", display: "flex", flexDirection: "column" }}>

      {/* Titlebar chrome */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, height: 36, padding: "0 14px", borderBottom: `1px solid ${T.divider}`, flexShrink: 0 }}>
        <span style={{ color: "var(--accent)", display: "flex", fontSize: 13 }}>⬡</span>
        <span style={{ fontSize: 11.5, color: T.t2 }}>Aeropeks Settings</span>
        <span style={{ flex: 1 }} />
        <span style={{ color: T.t3, display: "flex", gap: 14, alignItems: "center" }}>
          <span style={{ width: 9, height: 1.4, background: "currentColor", alignSelf: "center", display: "block" }} />
          <span style={{ width: 8, height: 8, border: "1.3px solid currentColor", borderRadius: 1.5, display: "block" }} />
          <span style={{ fontSize: 10, cursor: "pointer", lineHeight: 1 }} onClick={() => getCurrentWindow().hide()}>✕</span>
        </span>
      </div>

      {/* Scrollable content */}
      <div style={{ flex: 1, overflowY: "auto", padding: "24px 24px 0" }}>

        {/* Page heading */}
        <div style={{ marginBottom: 20 }}>
          <div style={{ fontSize: 19, fontWeight: 700, letterSpacing: "-0.01em" }}>Settings</div>
          <div style={{ fontSize: 11.5, color: T.t3, marginTop: 4 }}>Configure your personalized desktop menu bar.</div>
        </div>

        {/* SSH Shortcuts */}
        <SetSection
          title="System Shortcuts"
          action={
            <span
              style={{ fontSize: 11.5, fontWeight: 600, color: "var(--accent)", cursor: "pointer" }}
              onClick={addShortcut}
            >
              + Add shortcut
            </span>
          }
        >
          {shortcuts.length === 0 && (
            <span style={{ fontSize: 12, color: T.t3 }}>No shortcuts added yet.</span>
          )}
          {shortcuts.map((s) => (
            <div key={s.id} style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <Inp
                value={s.label}
                onChange={(e) => updateShortcut(s.id, "label", e.target.value)}
                placeholder="Label"
                style={{ flex: 2 }}
              />
              <Inp
                value={s.cmd}
                onChange={(e) => updateShortcut(s.id, "cmd", e.target.value)}
                placeholder="Command"
                style={{ flex: 3, fontFamily: "var(--font-mono)" }}
              />
              <Inp
                value={s.shortcut}
                onChange={(e) => updateShortcut(s.id, "shortcut", e.target.value)}
                placeholder="Hotkey"
                style={{ flex: 1 }}
              />
              <span
                onClick={() => removeShortcut(s.id)}
                style={{ color: T.t3, cursor: "pointer", flexShrink: 0, padding: "0 2px", lineHeight: 1, fontSize: 11 }}
              >
                ✕
              </span>
            </div>
          ))}
        </SetSection>

        {/* External Services */}
        <SetSection title="External Services">
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
            <SetField
              label="Weather Location"
              help={weatherLocation
                ? `Current: ${weatherLocation} (${weatherLat?.toFixed(2)}, ${weatherLon?.toFixed(2)})`
                : "Search your city for YR.no forecasts."}
            >
              <div style={{ position: "relative", flex: 1 }}>
                <Inp
                  value={searchQuery}
                  onChange={(e) => handleSearch(e.target.value)}
                  placeholder="Search for a city..."
                  style={{ flex: "none", width: "100%", boxSizing: "border-box" }}
                />
                {isSearching && (
                  <span style={{ position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)", width: 12, height: 12, borderRadius: 999, border: `2px solid ${T.t3}`, borderTopColor: "transparent", animation: "spin 0.8s linear infinite", display: "block" }} />
                )}
                {searchResults.length > 0 && (
                  <div style={{ position: "absolute", top: "100%", left: 0, right: 0, zIndex: 10, background: T.panelBg, border: T.panelBorder, borderRadius: T.cardR, marginTop: 4, overflow: "hidden", boxShadow: T.shadow }}>
                    {searchResults.map((loc, idx) => (
                      <div
                        key={idx}
                        onClick={() => selectLocation(loc)}
                        style={{ padding: "8px 12px", cursor: "pointer", display: "flex", justifyContent: "space-between", alignItems: "center", borderBottom: idx < searchResults.length - 1 ? `1px solid ${T.divider}` : "none" }}
                      >
                        <span style={{ fontSize: 12, color: T.t1 }}>{loc.name}</span>
                        <span style={{ fontSize: 11, color: T.t3 }}>{loc.country}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </SetField>

            <SetField label="GitHub Personal Access Token" help="Powers the Projects view. Needs repository read access.">
              <Inp
                type="password"
                value={githubToken}
                onChange={(e) => setGithubToken(e.target.value)}
                autoComplete="new-password"
                placeholder="github_pat_..."
              />
            </SetField>

            <SetField label="Usage Limits Service URL" help="Local AI usage tracking endpoint. Leave blank to disable.">
              <Inp
                value={usageLimitsUrl}
                onChange={(e) => setUsageLimitsUrl(e.target.value)}
                placeholder="http://localhost:8765/api/v1/snapshot"
                style={{ fontFamily: "var(--font-mono)" }}
              />
            </SetField>

            <SetField label="OBS Studio WebSocket" help="Enable 'WebSocket Server' in OBS Studio for live status tracking.">
              <Inp
                value={obsUrl}
                onChange={(e) => setObsUrl(e.target.value)}
                placeholder="ws://localhost:4455"
                style={{ flex: 2, fontFamily: "var(--font-mono)" }}
              />
              <Inp
                type="password"
                value={obsPassword}
                onChange={(e) => setObsPassword(e.target.value)}
                autoComplete="new-password"
                placeholder="Password"
                style={{ flex: 1 }}
              />
            </SetField>

            <SetField label="Dreame Robot Mower" help="Cloud credentials for mower status. Device ID from the Dreame app.">
              <div style={{ display: "flex", flexDirection: "column", gap: 6, flex: 1 }}>
                <div style={{ display: "flex", gap: 8 }}>
                  <Inp
                    value={dreameUsername}
                    onChange={(e) => setDreameUsername(e.target.value)}
                    placeholder="Dreame account email"
                    style={{ flex: 2 }}
                  />
                  <Inp
                    type="password"
                    value={dreamePassword}
                    onChange={(e) => setDreamePassword(e.target.value)}
                    autoComplete="new-password"
                    placeholder="Password"
                    style={{ flex: 1.5 }}
                  />
                </div>
                <Inp
                  value={dreameDeviceId}
                  onChange={(e) => setDreameDeviceId(e.target.value)}
                  placeholder="Device ID (e.g. -110196586)"
                  style={{ fontFamily: "var(--font-mono)" }}
                />
              </div>
            </SetField>

            <SetField label="Home Assistant" help="Shows camera and robot status. Token from HA Profile → Long-Lived Access Tokens.">
              <div style={{ display: "flex", flexDirection: "column", gap: 6, flex: 1 }}>
                <Inp
                  value={haUrl}
                  onChange={(e) => setHaUrl(e.target.value)}
                  placeholder="http://homeassistant.local:8123"
                  style={{ fontFamily: "var(--font-mono)" }}
                />
                <Inp
                  type="password"
                  value={haToken}
                  onChange={(e) => setHaToken(e.target.value)}
                  autoComplete="new-password"
                  placeholder="Long-lived access token"
                  style={{ fontFamily: "var(--font-mono)" }}
                />
              </div>
            </SetField>
          </div>

          <SetField label="Google Calendar Entity" help="HA entity ID for your Google Calendar. Find it in Developer Tools → States.">
            <Inp
              value={haCalendarEntityId}
              onChange={(e) => setHaCalendarEntityId(e.target.value)}
              placeholder="calendar.your_name_gmail_com"
              style={{ fontFamily: "var(--font-mono)" }}
            />
          </SetField>
        </SetSection>

        {/* Shell Companion */}
        <SetSection title="Shell Companion">
          <SetCheck
            on={reserveScreenSpace}
            onChange={setReserveScreenSpace}
            label="Reserve screen space for Aeropeks bars"
            help="Companion mode keeps Explorer alive. Turn this off if Windows work-area reservation gets weird."
          />
          <SetCheck
            on={hideNativeTaskbar}
            onChange={setHideNativeTaskbar}
            label="Hide the native Windows taskbar"
            help="Advanced replacement mode. Leave off unless you want Aeropeks to take over more shell surface."
          />
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <Btn onClick={handleRestoreShell}>Restore Windows Shell</Btn>
            <Btn onClick={handleClearIconCache}>Clear Icon Cache</Btn>
            <Btn onClick={() => invoke("open_demo_mode")}>Screenshot Mode</Btn>
          </div>
          {shellMessage && (
            <span style={{ fontSize: 11, color: T.t3 }}>{shellMessage}</span>
          )}
          <SetCheck
            on={debugInspector}
            onChange={setDebugInspector}
            label="Show window identity inspector"
            help="Reveals AUMID, relaunch command, process path, and icon source when an app shows the wrong icon."
          />
        </SetSection>

        {/* Debug inspector */}
        {debugInspector && (
          <SetSection title="Window Identity Snapshot">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span style={{ fontSize: 12, color: T.t2 }}>Current taskbar windows</span>
              <Btn onClick={refreshDebugWindows}>Refresh</Btn>
            </div>
            <div style={{ overflowX: "auto" }}>
              <table style={{ width: "100%", fontSize: 11, color: T.t2, borderCollapse: "collapse" }}>
                <thead>
                  <tr>
                    {["App", "Title", "Class", "AUMID", "Relaunch", "Path", "Icon", "Why"].map((h) => (
                      <th key={h} style={{ textAlign: "left", padding: "4px 6px", color: T.t3, fontWeight: 500, borderBottom: `1px solid ${T.divider}`, whiteSpace: "nowrap" }}>{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {debugWindows.map((win) => (
                    <tr key={win.hwnd}>
                      <td style={{ padding: "4px 6px" }}>{win.app_name || "—"}</td>
                      <td style={{ padding: "4px 6px", maxWidth: 120, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={win.title}>{win.title}</td>
                      <td style={{ padding: "4px 6px" }}>{win.class_name || "—"}</td>
                      <td style={{ padding: "4px 6px", maxWidth: 100, overflow: "hidden", textOverflow: "ellipsis" }} title={win.app_id ?? ""}>{win.app_id || "—"}</td>
                      <td style={{ padding: "4px 6px", maxWidth: 100, overflow: "hidden", textOverflow: "ellipsis" }} title={win.relaunch_command ?? win.relaunch_icon ?? ""}>{win.relaunch_command || win.relaunch_icon || "—"}</td>
                      <td style={{ padding: "4px 6px", maxWidth: 120, overflow: "hidden", textOverflow: "ellipsis" }} title={win.process_path ?? ""}>{win.process_path || "—"}</td>
                      <td style={{ padding: "4px 6px" }}>{win.icon_source || "—"}</td>
                      <td style={{ padding: "4px 6px" }}>{win.inclusion_reason}</td>
                    </tr>
                  ))}
                  {debugWindows.length === 0 && (
                    <tr>
                      <td colSpan={8} style={{ padding: "12px 6px", color: T.t3 }}>
                        Click Refresh to capture the current window list.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </SetSection>
        )}

        {/* Personalization */}
        <SetSection title="Personalization">
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
            <SetField label="Accent Color" help="Used for highlights and active indicators across the app.">
              <div style={{ display: "flex", alignItems: "center", gap: 8, flex: 1 }}>
                {SWATCHES.map((c) => (
                  <span
                    key={c}
                    onClick={() => { setAccentColor(c); document.documentElement.style.setProperty("--accent", c); }}
                    style={{
                      width: 22, height: 22, borderRadius: 5, background: c, cursor: "pointer", flexShrink: 0,
                      outline: accentColor.toLowerCase() === c.toLowerCase() ? `2px solid ${T.t1}` : "2px solid transparent",
                      outlineOffset: 2,
                    }}
                  />
                ))}
                <input
                  type="color"
                  value={accentColor}
                  onChange={(e) => { setAccentColor(e.target.value); document.documentElement.style.setProperty("--accent", e.target.value); }}
                  style={{ width: 22, height: 22, borderRadius: 4, border: "none", background: "none", cursor: "pointer", padding: 0, flexShrink: 0 }}
                  title="Custom color"
                />
                <span style={{ fontSize: 10.5, fontFamily: "var(--font-mono)", color: T.t3, marginLeft: 4 }}>
                  {accentColor.toUpperCase()}
                </span>
              </div>
            </SetField>

            <SetField label="Clock Format" help="HH:MM (24h) or HH:MM AM/PM (12h).">
              <span style={{ display: "inline-flex", background: T.inputBg, border: T.inputBorder, borderRadius: T.ctlR, padding: 2 }}>
                {([["24-Hour", true], ["12-Hour", false]] as const).map(([label, val]) => (
                  <span
                    key={label}
                    onClick={() => setUse24h(val)}
                    style={{
                      padding: "5px 14px",
                      borderRadius: Math.max(T.ctlR - 2, 2),
                      fontSize: 11.5,
                      fontWeight: 600,
                      cursor: "pointer",
                      background: use24h === val ? "var(--accent)" : "transparent",
                      color: use24h === val ? "#10131a" : T.t2,
                    }}
                  >
                    {label}
                  </span>
                ))}
              </span>
            </SetField>
          </div>
        </SetSection>

      </div>

      {/* Sticky footer */}
      <div style={{ padding: "14px 24px", borderTop: `1px solid ${T.divider}`, display: "flex", justifyContent: "flex-end", gap: 10, background: T.panelBg, flexShrink: 0 }}>
        <Btn onClick={() => getCurrentWindow().hide()}>Close Without Saving</Btn>
        <Btn primary onClick={handleSave}>
          {saved ? "✓ Settings Applied" : "Save All Changes"}
        </Btn>
      </div>

    </div>
  );
}

export default Settings;
