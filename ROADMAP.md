# Aeropeks Roadmap — Optimization & Modularization

*Based on a full codebase analysis on 2026-06-11. Goal: make the app lean, and turn the
current "built for Morten" setup into user-selectable modules.*

## Analysis summary

### What the app is today

A Tauri v2 top bar with 8 windows (main bar, settings, expanded-player, terminal-panel,
launcher-panel, 3 demo windows). The bar renders 13 feature clusters; all data flows
through `useMenuBarModel` which polls 9 backend commands on fixed intervals (5s–10min).
Backend is ~4.5k lines of Rust across 11 modules, deny-by-default window authorization,
secrets in Credential Manager.

### Findings (ordered by impact)

1. **Orphaned taskbar/dock subsystem still running hot.** The `native_windows` module
   (~900 lines of `main.rs`) runs a background thread every **1.2s** doing
   `EnumWindows` + COM property-store reads + icon extraction, and a second thread
   polling the foreground window title every **1s**. Since the Terminal Precision
   redesign, **nothing in the UI consumes any of it** — no dock, no window list, no
   virtual-desktop pills, and `windowTitle` is no longer rendered. This is the single
   biggest idle CPU cost in the app. The commands (`get_open_windows`, `focus_window`,
   `get_window_thumbnail`, `close_window`, virtual desktop switching) are all dead from
   the frontend.
2. **~1,400 lines of dead CSS.** 200 of 240 classes in `index.css` are unreferenced —
   the old pre-redesign popover/dock/settings styles were never deleted in Phase 3.
3. **Dead Dreame cloud integration.** `dreame.rs` (219 lines), `DreameTokenCache`,
   the `dreame_get_status` command, three settings fields, a Credential Manager target,
   and a full Settings UI section — all superseded by the HA mower integration.
4. **Hardcoded personal entity IDs.** `camera.garage`, `vacuum.roberto` (+6 sensors),
   `lawn_mower.a1_pro` (+8 sensors), `sensor.pixel_9_pro_xl_*` (×7) are baked into
   `main.rs`. This is the main blocker for anyone else using the app.
5. **HA polling is chatty.** Idle steady-state is ~23 HTTP GETs/minute (vacuum 7×/30s,
   mower 9×/60s, phone 7×/60s) and every command invocation builds a fresh
   `reqwest::Client`. HA offers a bulk `/api/states` fetch and a WebSocket
   `subscribe_entities` push API — one connection could replace all of it.
6. **Volume popover ships fake data.** The output-device list (Speakers / Headphones /
   Monitor, "Realtek HD Audio") is hardcoded demo content, not real endpoints.
7. **Build weight.** `tokio = { features = ["full"] }`, `reqwest` `blocking` feature,
   and a broad `windows` feature list (several features only serve the orphaned
   taskbar code, e.g. `Storage_Xps` for `PrintWindow`).
8. **Media polls every 5s** as a "fallback" even though GSMTC change events already
   push updates; the fallback could be 30s+.

## Modularization concept

Every bar feature becomes a **module**: a unit with an `enabled` flag, its own config
(entity IDs, URLs), a bar item, a popover, and a settings card. The bar renders only
enabled modules; pollers only run for enabled modules.

Module inventory: `usage-limits`, `projects`, `media`, `weather`, `camera`, `vacuum`,
`mower`, `phone`, `calendar`, `obs`, `terminal`, `launcher`, plus core system items
(volume, bluetooth, mic, privacy, power, clock) that are always on.

---

## Phase 1 — Cleanup (remove dead weight first) ✅ DONE 2026-06-11

- [x] Delete the `native_windows` taskbar subsystem + both background threads
      (`get_open_windows`, thumbnails, focus/close, icon cache, virtual desktops,
      foreground-title thread, debug inspector UI). Decision: the dock was a bust;
      deleted outright — git history preserves it.
- [x] Delete `dreame.rs`, `DreameTokenCache`, `dreame_get_status`, the three
      `dreame_*` settings fields, and the Settings card. The old Credential
      Manager entry is purged once on next settings load.
- [x] Purge dead CSS from `index.css` (2,638 → ~440 lines).
- [x] Drop the hardcoded audio-device list in the volume popover. Real endpoint
      enumeration (`IMMDeviceEnumerator`) is a future module feature.
- [x] Trim Cargo: removed `winvd` + `md5` crates, `reqwest/blocking`, `tokio`
      full→macros+time, 11 unused `windows` features.
- [x] Lower the media fallback poll to 30s (GSMTC events remain the primary signal).

*Finding during Phase 1: the `skip_serializing` attributes on secret settings
fields (from the original hardening) were lost at some point. All runtime paths
still strip secrets via `without_secrets()` (disk, broadcast, non-settings
windows), and the settings window receives secrets over IPC by design. The
settings test now guards the `without_secrets()` contract instead.*

## Phase 2 — Module config schema

- [ ] Add `modules` to `AppSettings`: `{ [id]: { enabled: bool, ...config } }` with
      serde defaults; migrate existing flat fields (weather coords, HA url, etc.).
- [ ] Move all entity IDs into module config: camera entity, vacuum entity prefix,
      mower entity prefix, phone device slug, calendar entity.
- [ ] New Settings layout: a **Modules** section — one card per module with an
      enable toggle and its config fields.
- [ ] `useMenuBarModel`: only fetch + schedule intervals for enabled modules.

## Phase 3 — Frontend module registry

- [ ] Define a `BarModule` interface: `{ id, section, useData(), BarItem, Popover,
      SettingsCard, pollMs }`.
- [ ] Convert each feature to a module definition; `App.tsx` renders registry output
      per section (DEV TOOLS / ENVIRONMENT / HOME / PERSONAL).
- [ ] Replace the 12 popover `useState`/`ref` pairs with a single
      `openPopoverId` state + one click-away handler + one window-height effect.
- [ ] Optional: user-configurable module order within sections.

## Phase 4 — Backend optimization

- [ ] Shared `reqwest::Client` in managed state (connection reuse for all HTTP).
- [ ] Replace the three bespoke HA commands with one `get_ha_states(entity_ids)`
      command validated against the configured module entities, fetched via the bulk
      `/api/states` endpoint (1 request instead of N).
- [ ] Stretch: HA WebSocket `subscribe_entities` — backend pushes
      `ha-state-changed` events; frontend HA intervals go away entirely.
- [ ] Per-module poll cadence in module config.

## Phase 5 — Generalization & release

- [ ] First-run onboarding: pick modules, enter credentials.
- [ ] Strip remaining personal defaults (weather defaults to empty, not Oslo).
- [ ] README: module list, setup per module, screenshots (demo mode).
- [ ] Release pipeline: `tauri build` artifact in CI, versioned releases.

---

### Open decisions

- ~~**Taskbar/dock**: deleted in Phase 1 or kept as a module?~~ → Deleted (2026-06-11);
  it wasn't viable at this stage. Git history has it if ever revisited.
- **HA transport**: bulk REST polling (simple, Phase 4 baseline) vs WebSocket push
  (best, more moving parts). Roadmap does REST first, WebSocket as stretch.
