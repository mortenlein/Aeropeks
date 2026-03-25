import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

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
}

function Settings() {
  const [plexUrl, setPlexUrl] = useState("");
  const [plexToken, setPlexToken] = useState("");
  const [accentColor, setAccentColor] = useState("#22c55e");
  const [shortcuts, setShortcuts] = useState<TerminalShortcut[]>([]);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<AppSettings>("get_settings").then((s) => {
      setPlexUrl(s.plex_url || "");
      setPlexToken(s.plex_token || "");
      setAccentColor(s.accent_color || "#22c55e");
      setShortcuts(s.terminal_shortcuts || []);
    });
  }, []);

  const handleSave = async () => {
    await invoke("save_settings", {
      settings: {
        plex_url: plexUrl,
        plex_token: plexToken,
        accent_color: accentColor,
        terminal_shortcuts: shortcuts,
      },
    });
    // Live-apply the accent color
    document.documentElement.style.setProperty("--accent", accentColor);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const addShortcut = () => {
    const newId = `ssh-${Date.now()}`;
    setShortcuts([...shortcuts, { id: newId, label: "New Shortcut", cmd: "ssh user@host" }]);
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

      <div className="setting-section">
        <div className="section-header">
          <h4>Plex Integration</h4>
        </div>

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
        <div className="section-header">
          <h4>Terminal Shortcuts</h4>
          <button className="text-button" onClick={addShortcut}>+ Add</button>
        </div>
        
        <div className="shortcut-list">
          {shortcuts.map((s) => (
            <div key={s.id} className="shortcut-item">
              <div className="shortcut-inputs">
                <input
                  type="text"
                  value={s.label}
                  onChange={(e) => updateShortcut(s.id, "label", e.target.value)}
                  placeholder="Label"
                  className="small"
                />
                <input
                  type="text"
                  value={s.cmd}
                  onChange={(e) => updateShortcut(s.id, "cmd", e.target.value)}
                  placeholder="Command"
                  className="small"
                />
              </div>
              <button 
                className="delete-button" 
                onClick={() => removeShortcut(s.id)}
                title="Delete"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="setting-section">
        <div className="section-header">
          <h4>Appearance</h4>
        </div>

        <div className="setting-group">
          <label>Accent Color</label>
          <div className="color-row">
            <input
              type="color"
              value={accentColor}
              onChange={(e) => {
                setAccentColor(e.target.value);
                document.documentElement.style.setProperty("--accent", e.target.value);
              }}
              className="color-picker"
            />
            <span className="setting-hint" style={{ marginLeft: 8 }}>{accentColor}</span>
          </div>
          <span className="setting-hint">Color of the accent bar and highlights</span>
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
