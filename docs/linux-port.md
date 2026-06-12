# Linux Port Spec (CachyOS / Arch, KDE Plasma)

Goal: make Aeropeks build and run on Linux behind `cfg` gates, with Windows
behavior unchanged. Target environment is CachyOS with KDE Plasma on Wayland
(the default session); X11 is treated as a degraded fallback.

> **Reality check (2026-06-12):** the test laptop actually runs **COSMIC**
> (cosmic-comp via cosmic-greeter), not KDE. cosmic-comp implements
> wlr-layer-shell, so the bar plan is unchanged, but two findings differ from
> the KDE assumptions: (1) the GNOME keyring default collection stays locked
> after login, so Secret Service calls block on an unlock prompt — secrets
> must never load on the UI path (see §4); (2) KWallet-specific notes don't
> apply. Phases 0 and 1 are implemented; the layer-shell bar renders with an
> exclusive zone under cosmic-comp.

## Current state

The `windows` crate is an unconditional dependency, so the project does not
compile on Linux at all. The Windows-specific surface is concentrated in five
modules; everything else is already portable.

| Module | Windows API used | Portable? |
|---|---|---|
| `shell.rs` | SHAppBarMessage, EnumWindows, DWM, SPI_SETWORKAREA | No — full rewrite |
| `system_status.rs` | WASAPI (volume/mic), GetSystemPowerStatus, PowerShell (camera, BT) | No — full rewrite |
| `media.rs` + `main.rs` listener | GSMTC (WinRT Media.Control) | Partially — Plex path is portable |
| `settings.rs` | Credential Manager (CredRead/Write/DeleteW) | Partially — file I/O is portable |
| `launcher.rs` | Start Menu `.lnk` scan, shutdown.exe, SetSuspendState, LockWorkStation | Partially — search UI/flow is portable |
| `terminal.rs` | pwsh.exe selection only (portable-pty is cross-platform) | Yes — 5-line shell pick |
| `ha.rs`, `integrations.rs`, `projects.rs`, `shortcuts.rs`, `http.rs`, `security.rs`, frontend | — | Yes — untouched |

## Architecture

Introduce a platform layer instead of sprinkling `cfg` through every function:

```
src-tauri/src/platform/
    mod.rs        # cfg re-exports
    win.rs        # current code moved here (shell, audio, power, creds, …)
    linux.rs      # new implementations
```

(`win.rs` rather than `windows.rs`, so the module name never shadows the
`windows` crate in import paths.)

Each platform file exposes the same function set:

- `configure_bar(window, reserve_screen_space, shutdown)` / `restore(handle)`
- `set_native_taskbar_visible(bool)` (Linux: no-op, hide the setting in UI)
- `get_volume / set_volume / mic_muted / set_mic_muted`
- `battery_status() -> BatteryStatus`
- `bluetooth_status() -> BluetoothStatus`
- `local_media() -> Option<MediaInfo>` / `local_media_action(action)`
- `watch_local_media(handle)` (event subscription)
- `read_secret / write_secret / delete_secret`
- `app_search(query) -> Vec<SearchResult>` / `launch_app(path)`
- `run_power_action(action)`

The `#[tauri::command]` wrappers, validation (`security.rs`), state, and all
frontend code stay shared.

### Cargo.toml

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58.0", features = [ ...current list... ] }

[target.'cfg(target_os = "linux")'.dependencies]
gtk = "0.18"                          # matches tauri v2's gtk3 stack
gtk-layer-shell = { version = "0.8", features = ["v0_6"] }  # wlr-layer-shell bindings; v0_6 unlocks set_keyboard_mode/is_supported
zbus = "4"                            # UPower, BlueZ, logind, MPRIS
mpris = "2"                           # local media sessions (replaces GSMTC)
freedesktop-desktop-entry = "0.5"     # .desktop parsing for the launcher
```

Consider moving secrets to the cross-platform `keyring` crate (v3) on **both**
platforms — it wraps Credential Manager on Windows and Secret Service (KWallet
implements it) on Linux, and deletes ~80 lines of unsafe from `settings.rs`.
If Windows behavior must stay byte-identical, keep the existing code in
`platform/windows.rs` and use `keyring`/`secret-service` only on Linux.

## Module specs

### 1. Bar placement — `shell.rs` (the core, do first)

**Wayland (primary):** use the layer-shell protocol via `gtk-layer-shell`.
In `setup()`, before the main window is shown, grab the underlying GTK window
(`window.gtk_window()`) and apply:

```rust
gtk_layer_shell::init_for_window(&gtk_win);
gtk_layer_shell::set_layer(&gtk_win, Layer::Top);
gtk_layer_shell::set_anchor(&gtk_win, Edge::Top | Edge::Left | Edge::Right);
gtk_layer_shell::set_exclusive_zone(&gtk_win, BAR_HEIGHT); // if reserve_screen_space
```

This replaces, natively and event-free, **all** of the Windows machinery:

- `SHAppBarMessage` reserve/maintain loop → exclusive zone (compositor-managed)
- `nudge_stuck_windows` → unnecessary, compositor tiles around the zone
- fullscreen-game detection → unnecessary, fullscreen surfaces stack above the Top layer
- `SPI_SETWORKAREA` reset on quit → unnecessary, zone dies with the surface
- `WS_EX_NOACTIVATE` → `set_keyboard_mode(KeyboardMode::OnDemand)`

The 1 s maintenance thread in `configure_main_window` is Windows-only; the
Linux path is fire-and-forget. `restore()` becomes a no-op apart from state.

Constraint: layer-shell must be initialized **before the GTK window is
realized**. The main window is already `visible: false` in `tauri.conf.json`,
so apply layer-shell in `setup()` before the `window.show()` call.

**X11 (fallback, optional):** set `_NET_WM_STRUT_PARTIAL` +
`_NET_WM_WINDOW_TYPE_DOCK` on the GTK window. Ship Wayland-only first;
detect via `WAYLAND_DISPLAY` and fall back to plain always-on-top on X11.

**`hide_native_taskbar`:** implemented on COSMIC (no-op elsewhere). The
panel settings are plain RON files that cosmic-panel hot-reloads, so
`set_native_taskbar_visible(false)` flips `autohide` +
`exclusive_zone: false` on every Top-anchored panel entry in
`~/.config/cosmic/com.system76.CosmicPanel.{Entry}/v1/`. Originals are
backed up under `~/.aeropeks/cosmic-panel-backup/` before the first write
and replayed by `set_native_taskbar_visible(true)` — which `restore_bar`
calls on quit and `setup()` calls at startup when the setting is off, so a
crash while hidden heals on the next launch.

### 2. System status — `system_status.rs`

- **Volume / mic:** PulseAudio API via `libpulse-binding`, or — much simpler
  and robust on CachyOS (PipeWire is standard) — shell out to `wpctl`:
  `wpctl get-volume @DEFAULT_AUDIO_SINK@`, `wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.42`,
  `wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle`. Start with `wpctl`, swap for
  libpulse later if polling cost matters (bar polls volume anyway).
- **Battery:** read sysfs directly — `/sys/class/power_supply/BAT*/capacity`
  and `status` ("Charging"). No dependency, no D-Bus. `has_battery` = glob
  matched anything.
- **Bluetooth:** BlueZ over D-Bus (zbus): `org.bluez` ObjectManager →
  devices with `Connected=true`, report `Name`. Replaces the PowerShell
  Get-PnpDevice heuristic and is actually more reliable.
- **Privacy mode:** mic mute maps 1:1 (`wpctl set-mute @DEFAULT_AUDIO_SOURCE@ 1`).
  Camera disable has **no unprivileged equivalent** of `Disable-PnpDevice`
  (would need `modprobe -r uvcvideo` + polkit). Spec: Linux privacy mode is
  mic-only; return a descriptive note instead of failing, and hide the camera
  claim in the UI tooltip on Linux.

### 3. Media — `media.rs` + the `main.rs` GSMTC listener

Replace GSMTC with MPRIS (D-Bus), which every Linux player exposes
(Spotify, browsers, mpv, …):

- `get_gsmtc_media()` → enumerate `org.mpris.MediaPlayer2.*` names, prefer a
  `Playing` player (same scoring: Playing=10, Paused=5). Map
  `Metadata` → title/artist/album, `PlaybackStatus` → `is_playing`.
  Bonus over GSMTC: MPRIS exposes `mpris:artUrl`, `mpris:length`, and
  `Position` — so `thumbnail`, `duration_ms`, `view_offset_ms` can be real
  values for local media on Linux (they're zeroed on Windows today).
- `gsmtc_action()` → `PlayPause` / `Next` / `Previous` on the selected player.
- The `CurrentSessionChanged`/`SessionsChanged` listener in `main.rs:457-499`
  → zbus signal streams: `NameOwnerChanged` (players appearing/vanishing) and
  `PropertiesChanged` on each player. Same `media-change` emit. Keep the 30 s
  fallback poll as-is (it's portable).
- `source` stays `"gsmtc"` on Linux too (or rename to `"local"` in both —
  frontend contract change, one string).

Plex fetch/control and `select_active` are untouched.

### 4. Secrets — `settings.rs`

Swap the four `Cred*W` helpers (`read_secret`/`write_secret`/`restore_secret`)
for the `keyring` crate, service `"Aeropeks"`, account = existing target
strings (`Aeropeks/PlexToken`, …). On KDE this lands in KWallet via Secret
Service. The save/rollback orchestration in `save()` is portable and stays.
The retired-Dreame purge is Windows-only history — gate it `cfg(windows)`.

**Secrets must never load on the UI path.** If the Secret Service collection
is locked (normal on a session where PAM didn't unlock the keyring, e.g.
COSMIC via greetd), every read triggers an unlock prompt and the D-Bus call
blocks until the prompt is answered — possibly forever. `settings::load_file`
reads the JSON only and feeds window setup; the full `settings::load` (which
attaches secrets) runs on a background thread in `setup()` that updates
`SharedSettings`, wakes the HA poller, and emits `settings-changed`.

### 5. Launcher — `launcher.rs`

- **App search:** replace the Start-Menu `.lnk` walk with XDG `.desktop`
  scanning: `$XDG_DATA_HOME/applications`, each `$XDG_DATA_DIRS/applications`
  (defaults `/usr/share/applications`, `/usr/local/share/applications`).
  Parse with `freedesktop-desktop-entry`; match query against `Name` (and
  `Keywords` if cheap); skip `NoDisplay=true`/`Hidden=true`.
  `allowed_shortcut` analogue: launched path must be inside one of those
  roots and end in `.desktop`.
- **Launch:** `gtk-launch <desktop-id>` (in PATH on any GTK system) — avoids
  hand-parsing `Exec=` field codes.
- **Power actions:** `systemctl poweroff` / `systemctl reboot` /
  `systemctl suspend` and lock via `loginctl lock-session`. All work
  unprivileged on a normal logind seat. (Or logind D-Bus via zbus to avoid
  subprocesses — `systemctl` is fine for v1.)
- Drop `CommandExt`/`CREATE_NO_WINDOW` on Linux (gate the import; no console
  windows exist there).

### 6. Terminal — `terminal.rs`

`portable-pty` already works on Linux. Gate `powershell_command()`:
Linux probes `pwsh` then falls back to `$SHELL` (then `/bin/bash`). Skip the
oh-my-posh pwsh init line for non-pwsh shells.

### 7. Hotkeys, tray, autostart (polish phase)

- **Global shortcuts (Alt+Space, terminal shortcuts):**
  `tauri-plugin-global-shortcut` uses X11 key grabs and does not work under
  Wayland. Options: (a) ship without global hotkeys on Wayland v1 — the bar
  click paths all still work; (b) use the XDG GlobalShortcuts portal
  (`ashpd` crate) — KDE implements it and shows the bindings in System
  Settings. Spec: (a) for v1, (b) as follow-up.
- **Tray:** Tauri tray works on KDE via StatusNotifier; needs
  `libappindicator-gtk3` installed. No code change expected.
- **Autostart:** `.desktop` file in `~/.config/autostart/` (manual or
  tauri-plugin-autostart later).

### tauri.conf.json / frontend

- No config fork needed: `transparent` works under KWin compositing;
  `alwaysOnTop`/`skipTaskbar` are harmless no-ops on Wayland (layer-shell
  supersedes them for the main window; panels remain regular windows and
  still work).
- Frontend: add a `get_platform` command (or expose via settings payload) to
  hide Windows-only settings (`hide_native_taskbar`, camera claim in privacy
  mode). Everything else renders identically in webkit2gtk.

## Build prerequisites on CachyOS

```
sudo pacman -S --needed base-devel webkit2gtk-4.1 gtk3 libappindicator-gtk3 \
                 librsvg gtk-layer-shell openssl
rustup target list  # plain x86_64-unknown-linux-gnu, no cross target needed
```

## Phasing

| Phase | Scope | Outcome | Est. effort |
|---|---|---|---|
| 0 ✅ | Cargo target-gating, `platform/` split, Linux stubs (volume=0, battery=none, plain always-on-top bar) | `cargo build` + bar renders on CachyOS; HA/Plex/weather/OBS/projects/shortcuts/terminal all functional | ~1 day |
| 1 ✅ | Layer-shell bar (exclusive zone, restore, settings toggle) | Real appbar behavior | ~0.5–1 day |
| 2 | wpctl volume/mic, sysfs battery, BlueZ bluetooth, MPRIS media + listener | Bar fully live | ~1–2 days |
| 3 | keyring secrets, .desktop launcher, logind power actions, shell pick | Feature parity minus Windows-only items | ~1 day |
| 4 | GlobalShortcuts portal, X11 strut fallback, autostart, packaging (AppImage/AUR) | Polish | as desired |

Phase 0 alone gets a useful bar on the laptop, since the HA modules — the
main reason to run it there — are pure HTTP and already portable.

## Known gaps on Linux (accepted)

- `hide_native_taskbar`: works on COSMIC (panel autohide); no-op on other DEs.
- The bar's layer surface maps to the focused output at launch, so on
  multi-monitor it can land on either screen. Pin with
  `LayerShell::set_monitor` if this gets annoying.
- Privacy mode disables the mic only (no unprivileged camera kill switch).
- Global hotkeys absent on Wayland until the portal implementation lands.
- Multi-monitor: layer-shell anchors to one output; current code is
  primary-monitor-only on Windows too, so parity.
