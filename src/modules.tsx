import type { CSSProperties, ReactNode } from "react";
import { WeatherPopover } from "./WeatherPopover";
import { UsageLimitsPopover } from "./UsageLimitsPopover";
import { ProjectsPopover } from "./ProjectsPopover";
import { MowerPopover } from "./MowerPopover";
import { CameraPopover } from "./CameraPopover";
import { VacuumPopover } from "./VacuumPopover";
import { PhonePopover } from "./PhonePopover";
import { CalendarPopover } from "./CalendarPopover";
import { ShortcutsPanel } from "./Shortcuts";
import { BarChip, BarItem, MowerGlyph, TrayIcon } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import type { useMenuBarModel } from "./hooks/useMenuBarModel";
import type { LimitProvider, LimitsSnapshot } from "./contracts";

export type MenuBarModel = ReturnType<typeof useMenuBarModel>;

export type BarSection = "dev" | "environment" | "home" | "personal";

export interface PopoverCtx {
  open: boolean;
  toggle: () => void;
}

/**
 * A self-contained bar feature: when `visible`, the bar renders `item` inside
 * a popover anchor for its section, and `popover` while the anchor is open.
 */
export interface BarModuleDef {
  id: string;
  section: BarSection;
  anchorStyle?: CSSProperties;
  visible: (model: MenuBarModel) => boolean;
  item: (model: MenuBarModel, popover: PopoverCtx) => ReactNode;
  popover: (model: MenuBarModel, popover: PopoverCtx) => ReactNode;
}

// ── Usage limits ─────────────────────────────────────────────────────

// The bar chip tracks the 5-hour window; the weekly window only shows in the popover.
function providerPct(p: LimitProvider): number {
  const v = p.shortWindow.remainingPercent ?? p.longWindow.remainingPercent;
  return v == null ? 100 : Math.round(v);
}

function chipTag(key: string): string {
  if (key.toLowerCase().includes("codex")) return "CDX";
  if (key.toLowerCase().includes("claude")) return "CLD";
  return key.slice(0, 3).toUpperCase();
}

function visibleProviders(model: MenuBarModel): Array<[string, LimitProvider]> {
  const snapshot: LimitsSnapshot | null = model.usageLimits;
  if (!snapshot) return [];
  const hidden = model.settings?.usage_hidden_providers ?? [];
  return Object.entries(snapshot.providers).filter(
    ([key, p]) => p.enabled && !hidden.includes(key),
  );
}

const usageLimitsModule: BarModuleDef = {
  id: "usage-limits",
  section: "dev",
  anchorStyle: { gap: 5 },
  visible: (m) => visibleProviders(m).length > 0,
  item: (m, { toggle }) =>
    visibleProviders(m).map(([key, p]) => (
      <BarChip key={key} tag={chipTag(key)} pct={providerPct(p)} onClick={toggle} />
    )),
  popover: (m) => m.usageLimits && <UsageLimitsPopover snapshot={m.usageLimits} />,
};

// ── Projects ─────────────────────────────────────────────────────────

const projectsModule: BarModuleDef = {
  id: "projects",
  section: "dev",
  anchorStyle: { gap: 5 },
  visible: (m) => m.projects !== null,
  item: (m, { toggle }) =>
    m.projects && (
      <>
        <BarItem
          icon={<Icon name="branch" size={12} />}
          hue="var(--accent)"
          mono={String(m.projects.projects.length)}
          onClick={toggle}
        />
        {m.projects.attentionCount > 0 && (
          <span style={{ padding: '1.5px 6px', borderRadius: 4, background: `color-mix(in srgb, ${HUE.red} 18%, transparent)`, border: `1px solid color-mix(in srgb, ${HUE.red} 35%, transparent)` }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600, color: HUE.red }}>{m.projects.attentionCount}</span>
          </span>
        )}
      </>
    ),
  popover: (m) =>
    m.projects && (
      <ProjectsPopover
        snapshot={m.projects}
        refreshing={m.projectsRefreshing}
        onRefresh={() => m.refreshProjects().catch(console.error)}
      />
    ),
};

// ── Shortcuts ────────────────────────────────────────────────────────
// One quiet tray item; favicons live in the dropdown only (spec rule).

const shortcutsModule: BarModuleDef = {
  id: "shortcuts",
  section: "dev",
  visible: () => true,
  item: (_m, { open, toggle }) => (
    <TrayIcon icon={<Icon name="extlink" size={11} />} state={open ? "open" : "idle"} onClick={toggle} />
  ),
  popover: (m) => <ShortcutsPanel shortcuts={m.settings?.pinned_shortcuts ?? []} />,
};

// ── Weather ──────────────────────────────────────────────────────────

const weatherModule: BarModuleDef = {
  id: "weather",
  section: "environment",
  visible: (m) => m.weather !== null,
  item: (m, { toggle }) =>
    m.weather && (
      <BarItem
        icon={<Icon name="cloud" size={12} />}
        hue={HUE.weather}
        mono={`${Math.round(m.weather.temp)}°`}
        onClick={toggle}
      />
    ),
  popover: (m, { toggle }) => m.weather && <WeatherPopover data={m.weather} onClose={toggle} />,
};

// ── Camera ───────────────────────────────────────────────────────────

function cameraLabel(model: MenuBarModel): string {
  const entity = model.settings?.modules.camera.entity_id;
  if (!entity) return "Camera";
  return entity
    .split(".")[1]
    .replace(/_/g, " ")
    .replace(/^./, (c) => c.toUpperCase());
}

const cameraModule: BarModuleDef = {
  id: "camera",
  section: "home",
  visible: (m) =>
    !!(
      m.settings?.homeassistant_url &&
      m.settings.modules.camera.enabled &&
      m.settings.modules.camera.entity_id
    ),
  item: (m, { toggle }) => (
    <BarItem
      icon={<Icon name="cam" size={12} />}
      hue={T.t3}
      text={cameraLabel(m)}
      onClick={toggle}
    />
  ),
  popover: (m) => <CameraPopover label={cameraLabel(m)} />,
};

// ── Mower ────────────────────────────────────────────────────────────

const mowerModule: BarModuleDef = {
  id: "mower",
  section: "home",
  visible: (m) => m.mower !== null,
  item: (m, { toggle }) => {
    if (!m.mower) return null;
    const gc = m.mower.state === 'mowing' ? HUE.ok
      : m.mower.state === 'error' ? HUE.red
      : HUE.mower;
    return (
      <BarItem
        icon={<MowerGlyph size={13} color={gc} />}
        text={m.mower.state_label}
        onClick={toggle}
      />
    );
  },
  popover: (m) => m.mower && <MowerPopover mower={m.mower} />,
};

// ── Vacuum ───────────────────────────────────────────────────────────

const vacuumModule: BarModuleDef = {
  id: "vacuum",
  section: "home",
  visible: (m) => m.vacuum !== null,
  item: (m, { toggle }) => {
    if (!m.vacuum) return null;
    const vc = m.vacuum.cleaning ? HUE.ok
      : m.vacuum.status === 'charging' ? HUE.amber
      : HUE.vacuum;
    const vt = m.vacuum.cleaning
      ? `${m.vacuum.cleaning_progress}%`
      : m.vacuum.status.charAt(0).toUpperCase() + m.vacuum.status.slice(1);
    return (
      <BarItem
        icon={<Icon name={m.vacuum.charging ? "bolt" : "battery"} size={12} />}
        hue={vc}
        text={vt}
        onClick={toggle}
      />
    );
  },
  popover: (m) => m.vacuum && <VacuumPopover vacuum={m.vacuum} />,
};

// ── Phone ────────────────────────────────────────────────────────────

const phoneModule: BarModuleDef = {
  id: "phone",
  section: "personal",
  visible: (m) => m.phone !== null,
  item: (m, { toggle }) =>
    m.phone && (
      <BarItem
        icon={<Icon name={m.phone.charging ? "bolt" : "battery"} size={12} />}
        hue={HUE.phone}
        mono={`${m.phone.battery}%`}
        onClick={toggle}
      />
    ),
  popover: (m) => m.phone && <PhonePopover phone={m.phone} />,
};

// ── Calendar ─────────────────────────────────────────────────────────

const calendarModule: BarModuleDef = {
  id: "calendar",
  section: "personal",
  visible: (m) => m.calendar !== null,
  item: (m, { toggle }) => {
    if (m.calendar === null) return null;
    const now = new Date();
    const next = m.calendar.find(e => !e.all_day && new Date(e.end) > now);
    const ongoing = next && new Date(next.start) <= now;
    const calHue = ongoing ? 'var(--accent)' : next ? HUE.cal : undefined;
    const calTime = next
      ? new Date(next.start).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: false })
      : undefined;
    const calText = next ? `${calTime} ${next.summary}` : undefined;
    return (
      <BarItem
        icon={<Icon name="cal" size={11} />}
        hue={calHue}
        text={calText}
        dim={!next}
        style={{ maxWidth: 200 }}
        onClick={toggle}
      />
    );
  },
  popover: (m) => m.calendar !== null && <CalendarPopover events={m.calendar} />,
};

export const BAR_MODULES: BarModuleDef[] = [
  usageLimitsModule,
  projectsModule,
  shortcutsModule,
  weatherModule,
  cameraModule,
  mowerModule,
  vacuumModule,
  phoneModule,
  calendarModule,
];
