# Handoff: Aeropeks "Terminal Precision" Theme

Unified design system for the Aeropeks desktop menu-bar app (Windows shell companion): the top bar, all 12 popups, and the Settings window.

## Overview

Aeropeks grew popup-by-popup and each surface had its own colors, radii, headers, and type. This handoff defines **one theme — "Terminal Precision"** — that every surface follows, plus the token system behind it. The goal when implementing: a user should not be able to tell which popup was built first.

## About the Design Files

The files in this bundle are **design references created in HTML/JSX** — prototypes showing intended look and behavior, **not production code to copy directly**. Recreate these designs in the Aeropeks codebase's existing environment (whatever UI stack the app uses — WPF/WinUI, Electron, Tauri, etc.) using its established patterns. The JSX is small and readable; treat it as the precise spec for measurements and styling.

Open `Aeropeks Spec — Terminal Precision.html` in a browser to see everything rendered (needs the whole folder; fonts load from Google Fonts).

## Fidelity

**High-fidelity.** Colors, typography, spacing, radii, and states are final. Recreate pixel-perfectly. The sample data (track names, repo names, weather) is illustrative.

---

## Design Tokens

### Theme surface (the "B" theme object in `aero-tokens.jsx`)

| Token | Value |
|---|---|
| Desktop / page bg | `#0b0c10` |
| Panel bg | `#0e1013` |
| Panel border | `1px solid rgba(255,255,255,0.11)` |
| Panel radius | `8px` |
| Panel shadow | `0 12px 32px rgba(0,0,0,0.45)` |
| Panel padding | `14px` |
| Card bg (inner grouping) | `rgba(255,255,255,0.02)` |
| Card border | `1px solid rgba(255,255,255,0.08)` |
| Card radius | `5px` |
| Text 1 (primary) | `rgba(228,232,236,0.92)` |
| Text 2 (secondary) | `rgba(160,170,182,0.7)` |
| Text 3 (dim/labels) | `rgba(118,128,144,0.5)` |
| Divider / hairline | `rgba(255,255,255,0.08)` |
| Row dividers | **yes** — hairlines between list rows |
| Control bg (hover/selected fill) | `rgba(255,255,255,0.05)` |
| Control radius | `4px` |
| Pill radius | `4px` (squared pills, not rounded-full) |
| Input bg / border | `rgba(0,0,0,0.32)` / `1px solid rgba(255,255,255,0.1)` |
| Bar bg | `rgba(12,14,17,0.98)` + bottom hairline |

### Color rules

1. **Domain hues** share one lightness + chroma — `oklch(0.74 0.13 H)` — only hue varies. **One hue per popup**; everything else neutral.

| Domain | Value |
|---|---|
| media | `oklch(0.74 0.14 55)` |
| charge/amber | `oklch(0.78 0.13 85)` |
| ok/green | `oklch(0.74 0.14 152)` |
| phone | `oklch(0.75 0.12 172)` |
| weather | `oklch(0.75 0.12 218)` |
| vacuum | `oklch(0.72 0.12 248)` |
| calendar | `oklch(0.71 0.12 276)` |
| mower | `oklch(0.71 0.13 302)` |
| alert/red | `oklch(0.66 0.19 25)` |

2. **Accent is user-set** (Settings → Personalization; default `#22C55E`, curated options `#22C55E / #38BDF8 / #A78BFA / #F4845F`). Dev tools (AI usage, Projects, Terminal) ride the accent instead of a fixed hue. Accent also drives: selection states, focus, live calendar event, active tray icons, primary buttons, toggles. Use `color-mix(in srgb, ACCENT 8–18%, transparent)` for tints.

3. **Quota bars color by what's LEFT** (they show remaining allowance): green normally, **amber ≤25% left, red ≤10% left** (`sevLeft()` in tokens). Battery/device bars keep the device hue instead.

### Typography

Sans for labels/names/sentences; **mono for ALL data** (numbers, times, counts, IDs, units). No exceptions — this is what makes the bar scannable.

User-selectable pairing (Settings → Font Pairing), curated set:
- `Space Grotesk` + `JetBrains Mono` (default)
- `IBM Plex Sans` + `IBM Plex Mono`
- `Sora` + `Spline Sans Mono`

| Role | Spec | Used for |
|---|---|---|
| Display | sans 46/600, letter-spacing −3% | weather temp |
| Title | sans 13/600 | (A-theme headers; rarely used in B) |
| Label | sans 12.5/600 | row titles, event names |
| Body | sans 12/400, text-2 | helper text |
| Data | mono 11–12/500–600 | values, times |
| Micro | mono 9.5/upper, letter-spacing +12%, text-3 | section labels, captions |

### Spacing

4px base grid · panel pad 14 (B) · card pad 10–12 · header gap 12 · row height 28–34 · cluster gap in bar 8–10.

---

## Components

### Panel header (the B convention)
Mono uppercase micro-label in the **popup's domain hue**, left-aligned; right side holds optional actions then a mono `✕` in text-3. A hairline divider runs under the header (padding-bottom 9, margin-bottom 12).

### Status pill
Squared (4px radius), `1px` border at 40% of hue, fill at 9% of hue, 6px dot in hue with `0 0 6px` glow, label sans 11.5/600 in hue. **Tint = domain hue; the dot pulses (2.2s opacity 1→0.35) only while a process runs** (charging, mowing, recording). Examples: Docked (mower purple), Charging (vacuum blue, pulsing), Home (phone teal).

### Progress bar (PBar)
Track 4px (3px in bar chips), radius half-height, `rgba(255,255,255,0.07)`; fill in the given hue.

### Tray icon states (one glyph, four states, color only)
| State | Treatment | Meaning |
|---|---|---|
| Idle | text-3 | feature off/quiet |
| Active | **accent** | feature on (BT connected, etc.) |
| Open | text-1 on control fill (4px radius chip) | its popup is showing |
| Alert | red + 4px pulsing red dot top-right | needs attention (hot mic, error) |

Hover adds the control fill without changing color. Red is reserved for things that genuinely need the user.

### SourceTag
Provenance chip for media: mono 8–8.5px uppercase, letter-spacing +14%, text-3, 1px hairline border, fill `rgba(255,255,255,0.025)`, 4px radius. Text only ("PLEX", "SPOTIFY", "YOUTUBE") — no third-party logos.

### Stat cell
Mono 12.5/600 value over micro 8.5 caption, centered, grouped in a Card row (e.g. mower: 84 sessions / 1.28 ha / 608h).

---

## Top bar

32px strip, bar bg + bottom hairline. Items grouped into **clusters separated by 1×12px hairline dividers**:

`DEV TOOLS` (CDX + CLD quota chips · git icon + repo count + red attention badge) — center `NOW PLAYING` (prev/pause/next in text-3/text-2 · music note in media hue · artist sans 11.5 text-2 · title sans 11.5/600 text-1 · SourceTag) — `ENVIRONMENT` (weather glyph in weather hue + mono temp) — `HOME` (camera "Garage" · mower glyph + "Docked" · bolt + "Charging") — `PERSONAL` (battery + mono % · calendar + next event) — `SYSTEM` (tray icons w/ states) — `CLOCK` (mono 11/600).

Rules: icons carry the domain hue at ~90%; words stay neutral text-2; values mono text-1. Quota chips: hairline-bordered chip with mono tag (text-3), 26px mini bar, mono value colored by `sevLeft`.

The mower glyph is a tinted CSS-mask of `icons/mower-glyph.png` (white-on-transparent, generated from the owner's line drawing — thin strokes don't survive 14px, so it's a solid silhouette).

## Popups (all follow Panel + header convention)

- **AI Usage Limits** (320w): two Cards (Codex + badge "FREE", Claude); rows `5h/7d` → label mono 10, PBar, mono % colored by `sevLeft`, reset countdown mono 9.5 text-3.
- **Now Playing** (560w, media hue): 92px album art (rounded 5), title 16/700, artist 12.5 text-2, album micro, time-mono + 3px media-hue progress, prev/next + 40px square-ish play button filled media hue w/ colored shadow. SourceTag in header.
- **Weather** (380w, weather hue): display temp + location + condition row; "HOURLY" micro in hue → Card with 6 columns (mono hour, hue glyph, mono temp); "NEXT 7 DAYS" → hairline rows: day sans 12 (today 600/text-1), glyph text-3, mono hi/lo (lo text-3), precip mono text-3.
- **Projects** (360w, accent): search input row + "44 flagged" mono; repo rows: attention count mono 13/700 (red ≥40 else amber), name 12.5/600 + "private" micro tag, mono `issues/pulls/releases` counts, ago right, ext-link icon.
- **Calendar** (340w, calendar hue): day-group micro labels; event rows: mono time-range (76px col), name 12.5/600, location 10.5 text-3. **Live event**: 2px accent left bar + accent 8% fill, time in accent.
- **Camera** (460w): header "GARAGE" + pulsing red LIVE micro; 240px 16:9 feed, mono timestamp chip overlay top-left on `rgba(0,0,0,0.55)` blur.
- **Device cards** (mower 264w / vacuum 264w / phone 248w): product photo (contain, transparent png/webp — bundled in `uploads/`), status pill overlapping photo bottom (−14px), then KV rows (Online wifi-green, Battery + hue PBar + mono %, Map, firmware footer behind hairline).
- **Volume** (300w): slider row (speaker icon = mute, accent-filled track, 13px knob text-1, mono value) + "OUTPUT" micro → device rows: icon, name sans 12 (active 600/text-1 on control fill, accent icon + accent glow dot), connection sub in mono 9 text-3.
- **Power menu** (172w): 4 rows icon + sans 12.5; hover = control fill; Shut Down in red.
- **Terminal** (560w, accent): header w/ Reset (neutral) + Kill (red) ghost buttons; body `rgba(0,0,0,0.35)` + hairline, mono 11.5/1.75 — username accent, path vacuum-blue, branch mower-purple, accent block cursor blinking 1.2s.
- **Settings window** (880w): 36px titlebar (chip icon accent + name text-2 + window controls text-3); 19/700 page title; section Cards (pad 18) with **accent micro section labels**; fields = label sans 12/600 + input (input bg/border, mono for URLs/tokens) + helper 10.5 text-3 (calm — no colored words mid-sentence); shortcut rows = name + mono command + hotkey + red ✕; Personalization = 4 accent swatches (selected: 2px text-1 outline offset 2), segmented clock control (active = accent fill, dark text), font-pairing select; footer = ghost button + accent primary "Save All Changes" (dark text on accent).

## Interactions & Behavior

- Popups anchor under their bar item; click-away or ✕ dismisses. Bar item shows "open" state while its popup shows.
- Hover on rows/buttons: control fill (`rgba(255,255,255,0.05)`), no color change, ~0.1s.
- Pulse animation: opacity 1→0.35→1, 2.2s ease-in-out infinite — only on genuinely live things.
- Accent + font pairing are user settings; everything above must derive from CSS variables (`--accent`, `--font-ui`, `--font-mono`) so changes propagate instantly.

## State Management

- Per-popup open state (one open at a time) → drives tray "open" state.
- Service states: BT/Wi-Fi/mic/recording → idle/active/alert mapping above.
- Quota snapshots (Codex/Claude 5h+7d remaining % + reset time) → `sevLeft` thresholds 25/10.
- Media: source (plex/spotify/youtube), artist/title/album/art, position/duration, playing.

## Assets

- `icons/mower-glyph.png` — white-on-transparent silhouette mask for the bar (tint with mower hue via CSS mask).
- `uploads/049be956-….webp` — Dreame mower photo (transparent bg) — mower card.
- `uploads/roborocks7maxv-….webp` — Roborock S7 MaxV photo (transparent bg) — vacuum card.
- `uploads/pixel9.png` — Pixel 9 photo (transparent bg) — phone card.
- `uploads/dreamea1proicon.png` — original line drawing the glyph was derived from.
- Fonts from Google Fonts (weights 400–700).

## Files

- `screenshots/` — rendered reference shots: 01 tokens · 02/02b top bar + icon states · 03–05 popups · 06–07 settings. Use for visual diffing; the JSX + spec page remain the precise source.
- `Aeropeks Spec — Terminal Precision.html` — open this; the rendered spec page (tokens, bar, all popups, settings).
- `aero-tokens.jsx` — **the source of truth**: palette, theme object, sev/sevLeft, shared atoms (Panel, Card, Pill, PBar, Stat, KV, Micro/Mono, icons).
- `aero-bar.jsx` — top bar, BarChip, TrayIcon, SourceTag, MowerGlyph.
- `aero-popups-a.jsx` — AI usage, Projects, Media, Weather, Camera.
- `aero-popups-b.jsx` — device cards, Calendar, Volume, Power, Terminal.
- `aero-settings.jsx` — Settings window.
- `aero-spec.jsx` — the spec/token cards.
- `aero-final.jsx` — page assembly (+ Tweaks wiring showing how accent/fonts propagate).
- `image-slot.js`, `tweaks-panel.jsx` — prototype-only helpers; do not port.
