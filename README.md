# Aeropeks

A premium macOS-style top menu bar for Windows, built with Tauri 2, React, and Rust.

Aeropeks replaces the visual clutter of the Windows taskbar era with a single 40px bar at the top of your screen: system status, now playing, weather, smart-home status, pinned shortcuts, and a drop-down terminal — all configurable as independent modules you can switch on or off.

![The Aeropeks bar](docs/screenshots/bar.png)

## Modules

Everything outside the core system tray is a module. Open **Settings → Bar Modules** to choose what your bar shows; disabled modules cost nothing (no polling, no UI).

| Module | What it shows | Configuration |
| --- | --- | --- |
| Shortcuts | Up to 8 pinned websites with their real favicons in a dropdown | Managed from the bar — paste a URL, optionally name it, edit inline |
| Now Playing | Track + media controls (Windows GSMTC, with Plex fallback) | Optional Plex URL + token |
| Weather | Current temp in the bar, hourly/daily forecast popover (met.no) | Search your city in Settings |
| AI Usage Limits | Per-provider chips tracking the 5-hour rate-limit window | URL of a usage-limits service endpoint |
| GitHub Projects | Repo count, attention flags, health-score popover | GitHub personal access token (repo read) |
| OBS Status | Recording/streaming indicator | OBS WebSocket URL + password |
| Camera | Live snapshot popover from any Home Assistant camera | HA camera entity (e.g. `camera.garage`) |
| Vacuum | State, battery, cleaning progress | HA vacuum entity (e.g. `vacuum.roberto`) |
| Lawn Mower | State, zone, stats, firmware-update badge | HA `lawn_mower.*` entity (+ optional update entity) |
| Phone | Battery, charging, home/away, activity | HA companion-app device slug (e.g. `pixel_9_pro_xl`) |
| Calendar | Next event in the bar, 7-day agenda popover | HA calendar entity |

Always available: volume slider, Bluetooth devices, mic mute, privacy mode (mic + camera block), battery, drop-down terminal with SSH shortcuts and global hotkeys, Alt+Space launcher, power menu, clock.

### Shortcuts

Pin the sites you live in — including self-hosted ones. Favicons are discovered the way a
browser does it: the backend fetches the site's homepage and follows its declared
`<link rel="icon">` (so dashboards like Frigate or Home Assistant get their real icons),
falling back to `/favicon.ico` on the same origin and finally Google's favicon service for
public sites. Icons are cached to disk, fetched entirely through the backend (the webview's
CSP blocks external images), and a pinned site can only ever be opened by its saved,
validated entry — never an arbitrary URL.

### Home Assistant integration

Set your HA URL and a long-lived access token in Settings. A single background poller fetches HA's bulk `/api/states` endpoint (default every 30 s, configurable 5–600 s) and derives all module statuses from one response — entity IDs are validated and never interpolated unchecked into requests.

The vacuum and mower modules expect companion sensors named after the main entity's object id (`sensor.<object>_battery`, `sensor.<object>_cleaning_progress`, …), which is the naming HA integrations such as the Dreame integration produce by default. The phone module uses the HA companion app's standard sensor names for a device slug.

## First run

On a fresh install (no `~/.aeropeks/settings.json` yet) the Settings window opens automatically. Pick your modules, paste your tokens, save — the bar reconfigures live, no restart needed.

## Security model

- Every Tauri command is deny-by-default authorized per window (`security::require_window`).
- Secrets (Plex token, OBS password, GitHub token, HA token) live in **Windows Credential Manager**, never on disk; the settings file holds only non-sensitive config and is written atomically with backup/rollback.
- Entity IDs, coordinates, volume, and PTY dimensions are validated at the command boundary.
- Webview capabilities are split per window (main / panels / preferences).

## Tech stack

- **Frontend**: React + TypeScript + Vite — bar features defined in a module registry (`src/modules.tsx`), design tokens + atoms (`src/tokens.ts`, `src/atoms.tsx`)
- **Backend**: Rust (Tauri 2) — Win32 APIs for audio, Bluetooth, DWM app-bar reservation, and shell integration
- **Notable crates**: `reqwest` (one shared client for all integrations), `windows`, `obws`, `portable-pty`

## Building

### Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [Rust](https://www.rust-lang.org/) (stable)
- [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) for Windows

### Development

```bash
npm install
npm run tauri dev
```

### Production build

```bash
npm run tauri build
```

Pushing a `v*` tag builds NSIS + MSI installers and publishes a GitHub release via CI.

## Configuration

Settings live in `~/.aeropeks/settings.json`; secrets in Windows Credential Manager. Open Settings via the gear icon in the bar or the tray menu. Screenshot mode (Settings → Shell Companion) poses every panel for documentation shots.

## Notes

- **Requires Windows 10/11** — uses Win32 APIs throughout.
- The development roadmap and architecture decisions are tracked in [ROADMAP.md](ROADMAP.md).
