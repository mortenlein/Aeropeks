# Aeropeks

A premium macOS-style top menu bar for Windows, built with Tauri 2, React, and Rust.

Aeropeks brings the elegance and functionality of the macOS menu bar to Windows — a centralized hub for system status, media control, terminal access, and more, all in a sleek 32px bar that sits at the top of your screen.

## Features

- **System Status** — Battery, Bluetooth devices, microphone mute, volume slider
- **Media Control** — GSMTC integration with playback controls and Plex support
- **Weather** — Real-time conditions via met.no, click for hourly/daily detail
- **Dreame Robot Mower** — Live mower status (state, online/offline, firmware) via Dreame cloud API
- **AI Usage Limits** — Token/cost tracking dashboard
- **Git Projects** — Health scores and attention flags across local repos
- **Privacy Mode** — One-click mic + camera block via Windows privacy APIs
- **Integrated Terminal** — Drop-down panel with SSH shortcuts and global hotkeys
- **OBS Integration** — Recording/streaming status indicator
- **Virtual Desktops** — Navigate Windows desktops from the bar
- **Window Manager** — Taskbar with app icons, thumbnails, focus/close
- **Power Menu** — Lock, Sleep, Restart, Shutdown
- **Launcher** — Alt+Space app launcher with search
- **Screenshot / Demo Mode** — Full screenshot layout for promotion

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust (Tauri 2)
- **Styling**: Plain CSS (no framework)
- **Notable crates**: `reqwest`, `windows`, `winvd`, `obws`, `portable-pty`, `md5`

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

## Configuration

Settings are stored in `%APPDATA%\com.aeropeks.app\settings.json` (managed by Tauri). Sensitive credentials (Dreame password, OBS password) are stored in Windows Credential Manager — never in the settings file.

Open the Settings window via the gear icon in the bar.

## Notes

- **Dreame integration**: State and online status are fetched via the Dreame cloud REST API. Battery and mow stats are not available — they only flow via Alibaba AliFy MQTT which uses an encrypted auth mechanism that can't be replicated outside the official SDK.
- **Requires Windows 10/11** — uses Win32 APIs for audio, Bluetooth, DWM, and window management.
