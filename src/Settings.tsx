import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface TerminalShortcut {
  id: string;
  label: string;
  cmd: string;
  shortcut: string;
}

interface AppSettings {
  plex_url: string;
  plex_token: string;
  accent_color: string;
  terminal_shortcuts: TerminalShortcut[];
  weather_location: string;
  weather_lat: number | null;
  weather_lon: number | null;
  obs_websocket_url: string;
  obs_websocket_password: string;
  use_24h: boolean;
}

interface LocationResult {
  name: string;
  lat: number;
  lon: number;
  country: string;
  url_path: string;
}

function Settings() {
  const [plexUrl, setPlexUrl] = useState("");
  const [plexToken, setPlexToken] = useState("");
  const [accentColor, setAccentColor] = useState("#22c55e");
  const [shortcuts, setShortcuts] = useState<TerminalShortcut[]>([]);
  const [saved, setSaved] = useState(false);
  const [use24h, setUse24h] = useState(true);

  const [weatherLocation, setWeatherLocation] = useState("");
  const [weatherLat, setWeatherLat] = useState<number | null>(null);
  const [weatherLon, setWeatherLon] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<LocationResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  const [obsUrl, setObsUrl] = useState("");
  const [obsPassword, setObsPassword] = useState("");

  useEffect(() => {
    invoke<AppSettings>("get_settings").then((s) => {
      setPlexUrl(s.plex_url || "");
      setPlexToken(s.plex_token || "");
      setAccentColor(s.accent_color || "#22c55e");
      setShortcuts(s.terminal_shortcuts || []);
      setWeatherLocation(s.weather_location || "");
      setWeatherLat(s.weather_lat);
      setWeatherLon(s.weather_lon);
      setSearchQuery(s.weather_location || "");
      setObsUrl(s.obs_websocket_url || "");
      setObsPassword(s.obs_websocket_password || "");
      setUse24h(s.use_24h !== false);
    });
  }, []);

  const handleSearch = async (q: string) => {
    setSearchQuery(q);
    if (q.length < 3) {
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    try {
      const results = await invoke<LocationResult[]>("search_locations", { query: q });
      setSearchResults(results);
    } catch (e) {
      console.error("Search failed:", e);
    } finally {
      setIsSearching(false);
    }
  };

  const selectLocation = (loc: LocationResult) => {
    setWeatherLocation(loc.name);
    setWeatherLat(loc.lat);
    setWeatherLon(loc.lon);
    setSearchQuery(loc.name);
    setSearchResults([]);
  };

  const handleSave = async () => {
    await invoke("save_settings", {
      settings: {
        plex_url: plexUrl,
        plex_token: plexToken,
        accent_color: accentColor,
        terminal_shortcuts: shortcuts,
        weather_location: weatherLocation,
        weather_lat: weatherLat,
        weather_lon: weatherLon,
        obs_websocket_url: obsUrl,
        obs_websocket_password: obsPassword,
        use_24h: use24h,
      },
    });
    // Re-register hotkeys
    await invoke("register_hotkeys");
    
    // Live-apply the accent color
    document.documentElement.style.setProperty("--accent", accentColor);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const addShortcut = () => {
    const newId = `ssh-${Date.now()}`;
    setShortcuts([...shortcuts, { id: newId, label: "New Shortcut", cmd: "echo Hello", shortcut: "Alt+Shift+T" }]);
  };

  const removeShortcut = (id: string) => {
    setShortcuts(shortcuts.filter(s => s.id !== id));
  };

  const updateShortcut = (id: string, field: keyof TerminalShortcut, value: string) => {
    setShortcuts(shortcuts.map(s => s.id === id ? { ...s, [field]: value } : s));
  };

  return (
    <div className="settings-container">
      <h3>Aeropeks Settings</h3>
      <p className="setting-hint">Configure your personalized desktop menu bar experience.</p>

      <div className="setting-section">
        <h4>Media Integration</h4>
        <div className="setting-group">
          <label>Plex Server URL</label>
          <input
            type="text"
            value={plexUrl}
            onChange={(e) => setPlexUrl(e.target.value)}
            placeholder="http://192.168.1.100:32400"
          />
          <span className="setting-hint">The primary address of your Plex Media Server.</span>
        </div>

        <div className="setting-group">
          <label>Plex Token</label>
          <input
            type="password"
            value={plexToken}
            onChange={(e) => setPlexToken(e.target.value)}
            placeholder="Enter your Plex token"
          />
          <span className="setting-hint">Used for authentication and controlling playback.</span>
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
              placeholder="Password"
              style={{ flex: 1.5 }}
            />
          </div>
          <span className="setting-hint">Enable "WebSocket Server" in OBS Studio for live status tracking.</span>
        </div>
      </div>

      <div className="setting-section">
        <h4>Personalization</h4>
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
          <span className="setting-hint">This color will be used for high-visibility highlights across the app.</span>
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
          <span className="setting-hint">Choose between HH:MM (24h) or HH:MM AM/PM (12h) formats.</span>
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
