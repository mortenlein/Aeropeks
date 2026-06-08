# Aeropeks Architecture

## Runtime Boundary

Aeropeks is a Windows desktop application built with Tauri. React renders the
windows, while Rust owns operating-system access, credentials, process control,
media sessions, and shell integration.

Tauri remains the correct runtime because the product requires Win32, GSMTC,
Core Audio, Credential Manager, AppBar, global shortcuts, and PTY access. Those
capabilities belong behind Rust commands rather than in the webview.

## Backend Modules

- `security`: command authorization and input validation.
- `settings`: settings persistence and Windows Credential Manager.
- `media`: GSMTC/Plex discovery and backend-owned playback control.
- `system_status`: audio, battery, privacy, camera, and Bluetooth.
- `terminal`: PTY lifecycle and terminal event transport.
- `launcher`: allowlisted launcher and power actions.
- `integrations`: weather and OBS clients.
- `shell`: AppBar, work-area, and native taskbar restoration.
- `main`: window composition, tray/bootstrap, and the native window registry.

## Frontend Modules

Page components render individual Tauri windows. Stateful IPC orchestration
lives under `src/hooks`; serializable payload contracts live in
`src/contracts.ts`.

## Rules For Features

1. Every command must authorize its caller with `security::require_window`.
2. The backend owns privileged targets such as HWNDs, addresses, and sessions.
3. Secrets are never serialized to the settings JSON or emitted to normal
   windows.
4. Blocking OS or process calls must not run on an async executor thread.
5. New payload fields require a Rust serialization test and a matching
   TypeScript contract update.
6. New feature modules own their state and commands; `main.rs` only registers
   and composes them.
7. CI must pass before release: frontend tests/build/audit and Rust
   fmt/tests/Clippy.
