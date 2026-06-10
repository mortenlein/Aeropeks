import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsModel } from "./hooks/useSettingsModel";

function Settings() {
  const {
    accentColor, addShortcut, debugInspector, debugWindows,
    dreameDeviceId, dreamePassword, dreameUsername,
    githubToken, handleClearIconCache, handleRestoreShell, handleSave, handleSearch,
    hideNativeTaskbar, isSearching, obsPassword, obsUrl, plexToken, plexUrl,
    refreshDebugWindows, removeShortcut, reserveScreenSpace, saved, searchQuery,
    searchResults, selectLocation, setAccentColor, setDebugInspector,
    setDreameDeviceId, setDreamePassword, setDreameUsername,
    setGithubToken, setHideNativeTaskbar, setObsPassword, setObsUrl, setPlexToken, setPlexUrl,
    setReserveScreenSpace, setUsageLimitsUrl, setUse24h, shellMessage, shortcuts, updateShortcut,
    use24h, usageLimitsUrl, weatherLat, weatherLocation, weatherLon,
    haUrl, haToken, setHaUrl, setHaToken,
    haCalendarEntityId, setHaCalendarEntityId,
  } = useSettingsModel();

  return (
    <div className="settings-container">
      <h3>Aeropeks Settings</h3>
      <p className="setting-hint">Configure your personalized desktop menu bar experience.</p>

      <div className="setting-section">
        <h4>Media Integration</h4>
        <div className="setting-row-2col">
          <div className="setting-group">
            <label>Plex Server URL</label>
            <input
              type="text"
              value={plexUrl}
              onChange={(e) => setPlexUrl(e.target.value)}
              placeholder="http://192.168.1.100:32400"
            />
            <span className="setting-hint">Address of your Plex Media Server.</span>
          </div>
          <div className="setting-group">
            <label>Plex Token</label>
            <input
              type="password"
              value={plexToken}
              onChange={(e) => setPlexToken(e.target.value)}
              autoComplete="new-password"
              placeholder="Enter your Plex token"
            />
            <span className="setting-hint">Used for authentication and playback control.</span>
          </div>
        </div>
      </div>

      <div className="setting-section">
        <div className="section-header">
          <h4>System Shortcuts</h4>
          <button className="text-button" onClick={addShortcut}>+ Add Action</button>
        </div>
        
        <div className="shortcut-list">
          {shortcuts.map((s) => (
            <div key={s.id} className="shortcut-item">
              <div className="shortcut-inputs">
                <input
                  type="text"
                  value={s.label}
                  onChange={(e) => updateShortcut(s.id, "label", e.target.value)}
                  placeholder="Label (e.g. Git Status)"
                  className="small"
                  style={{ flex: 1.5 }}
                />
                <input
                  type="text"
                  value={s.cmd}
                  onChange={(e) => updateShortcut(s.id, "cmd", e.target.value)}
                  placeholder="Terminal Command"
                  className="small"
                  style={{ flex: 2 }}
                />
                <input
                  type="text"
                  value={s.shortcut}
                  onChange={(e) => updateShortcut(s.id, "shortcut", e.target.value)}
                  placeholder="Hotkey"
                  className="small"
                  style={{ flex: 1 }}
                />
              </div>
              <button 
                className="delete-button" 
                onClick={() => removeShortcut(s.id)}
                title="Remove action"
              >
                ×
              </button>
            </div>
          ))}
          {shortcuts.length === 0 && (
            <div className="setting-hint" style={{ textAlign: "center", padding: "10px" }}>
              No custom shortcuts added yet.
            </div>
          )}
        </div>
      </div>

      <div className="setting-section">
        <h4>External Services</h4>
        <div className="setting-group">
          <label>Weather Forecast Location</label>
          <div className="search-input-wrapper">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
              placeholder="Search for a city (e.g. Oslo, London...)"
            />
            {isSearching && <div className="spinner-small"></div>}
            {searchResults.length > 0 && (
              <div className="search-results-dropdown">
                {searchResults.map((loc, idx) => (
                  <div 
                    key={idx} 
                    className="search-result-item"
                    onClick={() => selectLocation(loc)}
                  >
                    <span className="result-name">{loc.name}</span>
                    <span className="result-country">{loc.country}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
          <span className="setting-hint">
            {weatherLocation ? (
              <>Current location: <strong>{weatherLocation}</strong> ({weatherLat?.toFixed(2)}, {weatherLon?.toFixed(2)})</>
            ) : (
              <>Search for your city to get accurate weather forecasts from YR.no.</>
            )}
          </span>
        </div>

        <div className="setting-row-2col">
          <div className="setting-group">
            <label>GitHub Personal Access Token</label>
            <input
              type="password"
              value={githubToken}
              onChange={(e) => setGithubToken(e.target.value)}
              autoComplete="new-password"
              placeholder="github_pat_..."
            />
            <span className="setting-hint">Powers the Projects view. Needs repository read access.</span>
          </div>
          <div className="setting-group">
            <label>Usage Limits Service URL</label>
            <input
              type="text"
              value={usageLimitsUrl}
              onChange={(e) => setUsageLimitsUrl(e.target.value)}
              placeholder="http://localhost:8765/api/v1/snapshot"
            />
            <span className="setting-hint">Local AI usage tracking endpoint. Leave blank to disable.</span>
          </div>
        </div>

        <div className="setting-group">
          <label>OBS Studio WebSocket</label>
          <div className="color-row" style={{ marginTop: 0 }}>
            <input
              type="text"
              value={obsUrl}
              onChange={(e) => setObsUrl(e.target.value)}
              placeholder="ws://localhost:4455"
              style={{ flex: 2 }}
            />
            <input
              type="password"
              value={obsPassword}
              onChange={(e) => setObsPassword(e.target.value)}
              autoComplete="new-password"
              placeholder="Password"
              style={{ flex: 1.5 }}
            />
          </div>
          <span className="setting-hint">Enable "WebSocket Server" in OBS Studio for live status tracking.</span>
        </div>

        <div className="setting-group">
          <label>Dreame Robot Mower</label>
          <div className="color-row" style={{ marginTop: 0 }}>
            <input
              type="text"
              value={dreameUsername}
              onChange={(e) => setDreameUsername(e.target.value)}
              placeholder="Dreame account email"
              style={{ flex: 2 }}
            />
            <input
              type="password"
              value={dreamePassword}
              onChange={(e) => setDreamePassword(e.target.value)}
              autoComplete="new-password"
              placeholder="Password"
              style={{ flex: 1.5 }}
            />
          </div>
          <input
            type="text"
            value={dreameDeviceId}
            onChange={(e) => setDreameDeviceId(e.target.value)}
            placeholder="Device ID (e.g. -110196586)"
            style={{ marginTop: '6px' }}
          />
          <span className="setting-hint">Dreame cloud credentials for live mower status in the menu bar. Device ID from the app or your router's ARP table.</span>
        </div>

        <div className="setting-group">
          <label>Home Assistant</label>
          <div className="color-row" style={{ marginTop: 0 }}>
            <input
              type="text"
              value={haUrl}
              onChange={(e) => setHaUrl(e.target.value)}
              placeholder="http://homeassistant.local:8123"
              style={{ flex: 2 }}
            />
            <input
              type="password"
              value={haToken}
              onChange={(e) => setHaToken(e.target.value)}
              autoComplete="new-password"
              placeholder="Long-lived access token"
              style={{ flex: 2 }}
            />
          </div>
          <span className="setting-hint">Shows the Garage camera in the menu bar. Token from HA Profile → Long-Lived Access Tokens.</span>
        </div>

        <div className="setting-group">
          <label>Google Calendar Entity</label>
          <input
            type="text"
            value={haCalendarEntityId}
            onChange={(e) => setHaCalendarEntityId(e.target.value)}
            placeholder="calendar.your_name_gmail_com"
          />
          <span className="setting-hint">HA entity ID for your Google Calendar. Find it in Developer Tools → States.</span>
        </div>
      </div>

      <div className="setting-section">
        <h4>Shell Companion</h4>
        <div className="setting-group">
          <label className="toggle-row">
            <input
              type="checkbox"
              checked={reserveScreenSpace}
              onChange={(e) => setReserveScreenSpace(e.target.checked)}
            />
            Reserve screen space for Aeropeks bars
          </label>
          <span className="setting-hint">
            Companion mode keeps Explorer alive. Turn this off if Windows work-area reservation gets weird; saving will restore the native work area.
          </span>
        </div>

        <div className="setting-group">
          <label className="toggle-row">
            <input
              type="checkbox"
              checked={hideNativeTaskbar}
              onChange={(e) => setHideNativeTaskbar(e.target.checked)}
            />
            Hide the native Windows taskbar
          </label>
          <span className="setting-hint">
            Advanced replacement mode. Leave this off unless you want Aeropeks to take over more shell surface.
          </span>
        </div>

        <div className="shell-actions">
          <button onClick={handleRestoreShell}>Restore Windows Shell</button>
          <button onClick={handleClearIconCache}>Clear Icon Cache</button>
          <button onClick={() => invoke("open_demo_mode")}>Screenshot Mode (closes this window)</button>
        </div>
        {shellMessage && <span className="setting-hint">{shellMessage}</span>}

        <div className="setting-group">
          <label className="toggle-row">
            <input
              type="checkbox"
              checked={debugInspector}
              onChange={(e) => setDebugInspector(e.target.checked)}
            />
            Show window identity inspector
          </label>
          <span className="setting-hint">
            Use this when an app shows the wrong icon. It reveals the AUMID, relaunch command, process path, and icon source Aeropeks picked.
          </span>
        </div>

        {debugInspector && (
          <div className="debug-inspector">
            <div className="section-header">
              <h4>Window Identity Snapshot</h4>
              <button className="text-button" onClick={refreshDebugWindows}>Refresh</button>
            </div>
            <div className="debug-window-table">
              <table>
                <thead>
                  <tr>
                    <th>App</th>
                    <th>Title</th>
                    <th>Class</th>
                    <th>AUMID</th>
                    <th>Relaunch</th>
                    <th>Path</th>
                    <th>Icon</th>
                    <th>Why included</th>
                  </tr>
                </thead>
                <tbody>
                  {debugWindows.map((win) => (
                    <tr key={win.hwnd}>
                      <td>{win.app_name || "-"}</td>
                      <td title={win.title}>{win.title}</td>
                      <td>{win.class_name || "-"}</td>
                      <td title={win.app_id || ""}>{win.app_id || "-"}</td>
                      <td title={win.relaunch_command || win.relaunch_icon || ""}>
                        {win.relaunch_command || win.relaunch_icon || "-"}
                      </td>
                      <td title={win.process_path || ""}>{win.process_path || "-"}</td>
                      <td title={win.identity_key}>{win.icon_source || "-"}</td>
                      <td>{win.inclusion_reason}</td>
                    </tr>
                  ))}
                  {debugWindows.length === 0 && (
                    <tr>
                      <td colSpan={8}>Click refresh to capture the current taskbar window identity list.</td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>

      <div className="setting-section">
        <h4>Personalization</h4>
        <div className="setting-row-2col">
          <div className="setting-group">
            <label>Theme Accent Color</label>
            <div className="color-row" style={{ marginTop: 0 }}>
              <input
                type="color"
                value={accentColor}
                onChange={(e) => {
                  setAccentColor(e.target.value);
                  document.documentElement.style.setProperty("--accent", e.target.value);
                }}
                className="color-picker"
              />
              <span className="setting-hint" style={{ fontWeight: 600 }}>{accentColor.toUpperCase()}</span>
            </div>
            <span className="setting-hint">Used for highlights and active indicators across the app.</span>
          </div>
          <div className="setting-group">
            <label>Clock Format</label>
            <div className="color-row" style={{ marginTop: 0 }}>
              <button
                className={use24h ? "primary" : ""}
                onClick={() => setUse24h(true)}
                style={{ flex: 1, padding: "8px" }}
              >
                24-Hour
              </button>
              <button
                className={!use24h ? "primary" : ""}
                onClick={() => setUse24h(false)}
                style={{ flex: 1, padding: "8px" }}
              >
                12-Hour
              </button>
            </div>
            <span className="setting-hint">HH:MM (24h) or HH:MM AM/PM (12h).</span>
          </div>
        </div>
      </div>

      <div className="settings-footer">
        <button onClick={() => getCurrentWindow().hide()}>Close Without Saving</button>
        <button className="primary" onClick={handleSave}>
          {saved ? "✓ Settings Applied" : "Save All Changes"}
        </button>
      </div>
    </div>
  );
}

export default Settings;
