import { Wifi, WifiOff, Cpu, Scissors, Clock, Map, Moon, AlertCircle } from "lucide-react";
import type { MowerStatus } from "./contracts";
import mowerImg from "./assets/mower.webp";

interface Props {
  mower: MowerStatus;
}

const STATE_COLOR: Record<string, string> = {
  mowing: "#4ade80",
  paused: "#fb923c",
  error:  "#ef4444",
  docked: "#a78bfa",
};

const isActive = (state: string) => state === "mowing";
const isOnline = (state: string) => state !== "unavailable" && state !== "";

function formatHours(minutes: number): string {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function formatArea(m2: number): string {
  return m2 >= 10000 ? `${(m2 / 10000).toFixed(2)} ha` : `${m2} m²`;
}

export function MowerPopover({ mower }: Props) {
  const stateColor = STATE_COLOR[mower.state] ?? "rgba(255,255,255,0.4)";
  const online = isOnline(mower.state);

  return (
    <div className="mower-popover">
      <div className="mower-popover-hero">
        <img src={mowerImg} alt="Dreame A1 Pro" className="mower-popover-img" />
        <div className="mower-popover-img-fade" />
        <div
          className={`mower-popover-state-badge ${isActive(mower.state) ? "mower-badge-pulse" : ""}`}
          style={{ color: stateColor, borderColor: stateColor }}
        >
          <span
            className="mower-state-dot"
            style={{ background: stateColor, boxShadow: `0 0 6px ${stateColor}` }}
          />
          {mower.state_label}
        </div>
      </div>

      <div className="mower-popover-body">
        <div className="mower-stat-row">
          {online
            ? <Wifi size={13} style={{ color: "#4ade80", flexShrink: 0 }} />
            : <WifiOff size={13} style={{ color: "#ef4444", flexShrink: 0 }} />}
          <span className="mower-stat-label">{online ? "Online" : "Offline"}</span>
          {mower.dnd && (
            <span title="Do Not Disturb" style={{ marginLeft: "auto", display: "flex" }}>
              <Moon size={12} style={{ color: "#a78bfa", flexShrink: 0 }} />
            </span>
          )}
          {mower.has_update && (
            <span title="Update available" style={{ marginLeft: mower.dnd ? 4 : "auto", display: "flex" }}>
              <AlertCircle size={12} style={{ color: "#fb923c", flexShrink: 0 }} />
            </span>
          )}
        </div>

        {mower.zone_state && mower.zone_state !== "unknown" && mower.zone_state !== "" && (
          <div className="mower-stat-row">
            <Map size={13} style={{ color: "rgba(255,255,255,0.35)", flexShrink: 0 }} />
            <span className="mower-stat-label">Zone {mower.zone_id} — {mower.zone_state}</span>
          </div>
        )}

        <div className="mower-stats-grid">
          <div className="mower-stat-cell">
            <Scissors size={11} style={{ color: "rgba(255,255,255,0.35)" }} />
            <span className="mower-stat-value">{mower.cleaning_count}</span>
            <span className="mower-stat-unit">sessions</span>
          </div>
          <div className="mower-stat-cell">
            <span className="mower-stat-value">{formatArea(mower.total_area_m2)}</span>
            <span className="mower-stat-unit">total area</span>
          </div>
          <div className="mower-stat-cell">
            <Clock size={11} style={{ color: "rgba(255,255,255,0.35)" }} />
            <span className="mower-stat-value">{formatHours(mower.total_time_min)}</span>
            <span className="mower-stat-unit">runtime</span>
          </div>
        </div>

        <div className="mower-stat-row" style={{ marginTop: 2 }}>
          <Cpu size={13} style={{ color: "rgba(255,255,255,0.35)", flexShrink: 0 }} />
          <span className="mower-stat-label" style={{ opacity: 0.4 }}>fw {mower.firmware}</span>
        </div>
      </div>
    </div>
  );
}
