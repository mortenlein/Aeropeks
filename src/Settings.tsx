import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface AppSettings {
  plex_url: string;
  plex_token: string;
}

function Settings() {
  const [plexUrl, setPlexUrl] = useState("");
  const [plexToken, setPlexToken] = useState("");
  const [height, setHeight] = useState(32);
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings").then((s) => {
      setPlexUrl(s.plex_url || "");
      setPlexToken(s.plex_token || "");
    });
  }, []);

  const handleSave = async () => {
    await invoke("save_settings", {
      settings: {
        plex_url: plexUrl,
        plex_token: plexToken,
      },
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="settings-container">
      <h3>Aeropeks Settings</h3>

      <div className="setting-section">
        <h4>Plex Integration</h4>

        <div className="setting-group">
          <label>Plex Server URL</label>
          <input
            type="text"
            value={plexUrl}
            onChange={(e) => setPlexUrl(e.target.value)}
            placeholder="http://192.168.1.100:32400"
          />
          <span className="setting-hint">Your Plex Media Server address</span>
        </div>

        <div className="setting-group">
          <label>Plex Token</label>
          <input
            type="password"
            value={plexToken}
            onChange={(e) => setPlexToken(e.target.value)}
            placeholder="Enter your Plex token"
          />
          <span className="setting-hint">Found in Plexamp Settings → About</span>
        </div>
      </div>

      <div className="setting-section">
        <h4>Appearance</h4>

        <div className="setting-group">
          <label>Bar Height (px)</label>
          <input
            type="number"
            value={height}
            onChange={(e) => setHeight(parseInt(e.target.value))}
          />
        </div>

        <div className="setting-group">
          <label>
            <input
              type="checkbox"
              checked={alwaysOnTop}
              onChange={(e) => setAlwaysOnTop(e.target.checked)}
            />
            Always on Top
          </label>
        </div>
      </div>

      <div className="settings-footer">
        <button onClick={() => getCurrentWindow().hide()}>Cancel</button>
        <button className="primary" onClick={handleSave}>
          {saved ? "✓ Saved!" : "Save Changes"}
        </button>
      </div>
    </div>
  );
}

export default Settings;
