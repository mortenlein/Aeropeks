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

## Phase 2 — Module config schema ✅ DONE 2026-06-11

- [x] `modules` struct in `AppSettings` (typed per-module config, serde defaults);
      legacy settings files are migrated on load — previously hardcoded entity ids
      are seeded, and the old flat `ha_calendar_entity_id` is carried over.
- [x] All entity IDs moved into module config: camera entity, vacuum entity
      (sensor names derived from its object id), mower entity + optional update
      entity, phone device slug, calendar entity. Entity ids/slugs are strictly
      validated (`security::validate_ha_entity_id` / `validate_ha_slug`) since
      they are interpolated into REST URL paths.
- [x] Settings: new **Bar Modules** section — toggles for media / weather /
      usage limits / projects / OBS, and toggle + entity inputs for the five
      HA modules.
- [x] `useMenuBarModel` split into a core effect (settings, volume, system
      statuses, event listeners) and a module effect that fetches + schedules
      intervals only for enabled, configured modules and tears down on toggle.

## Phase 3 — Frontend module registry ✅ DONE 2026-06-11

- [x] `BarModuleDef` interface in `src/modules.tsx`:
      `{ id, section, anchorStyle?, visible(model), item(model, ctx), popover(model, ctx) }`.
      Data fetching stays in `useMenuBarModel` (already module-gated since Phase 2).
- [x] All eight feature modules (usage limits, projects, weather, camera, mower,
      vacuum, phone, calendar) converted to registry definitions; `App.tsx`
      renders sections (DEV / ENVIRONMENT / HOME / PERSONAL) from the registry.
- [x] Twelve popover `useState`/`ref` pairs replaced with one `openPopover` id,
      one `data-popover-id` click-away handler, and one window-height effect.
      System-tray popovers (bluetooth, volume, terminal, power) share the same
      mechanism; screenshot mode forces its three popovers open via a flag.
- [ ] Optional (deferred): user-configurable module order within sections.

## Phase 4 — Backend optimization ✅ DONE 2026-06-11

- [x] Shared `reqwest::Client` (`http.rs`) in managed state — one connection pool
      for HA, weather, location search, usage limits, GitHub, and Plex
      (media/control/album art); per-request timeouts replace per-call clients.
- [x] New `ha.rs`: a backend poller fetches the bulk `/api/states` endpoint once
      per cycle and derives vacuum + mower + phone statuses from that single
      response (was ~23 entity GETs/min, now 2 bulk GETs/min at default cadence).
      The snapshot is cached (`get_ha_snapshot` serves it on startup) and pushed
      to the bar via `ha-snapshot` events — the three frontend HA intervals are
      gone. Saving settings nudges the poller (`tokio::sync::Notify`) so module
      changes apply immediately; HA outages keep the last snapshot instead of
      flickering items away. Calendar/camera remain on-demand commands using the
      shared client.
- [ ] Stretch (deferred): HA WebSocket `subscribe_entities` push transport.
- [x] Poll cadence: `homeassistant_poll_seconds` setting (default 30, clamped
      5–600) drives the poller; exposed in Settings. Per-module cadence was
      dropped — with one bulk fetch there is a single meaningful knob.

## Phase 5 — Generalization & release ✅ DONE 2026-06-11

- [x] First-run onboarding: a missing settings file opens the Settings window
      automatically so modules and credentials can be configured. (A guided
      wizard remains a possible future upgrade.)
- [x] Personal defaults stripped: weather defaults to unset (was Oslo), the
      "SSH: Home Lab" default terminal shortcut removed.
- [x] README rewritten: module table with per-module configuration, HA entity
      naming conventions, first-run behavior, security model, build + release.
- [x] Release pipeline already existed (`release.yml`: tag `v*` → NSIS + MSI
      via tauri-action) — verified, no changes needed.

---

### Open decisions

- ~~**Taskbar/dock**: deleted in Phase 1 or kept as a module?~~ → Deleted (2026-06-11);
  it wasn't viable at this stage. Git history has it if ever revisited.
- **HA transport**: bulk REST polling (simple, Phase 4 baseline) vs WebSocket push
  (best, more moving parts). Roadmap does REST first, WebSocket as stretch.
