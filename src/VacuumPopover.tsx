import { Battery, BatteryCharging, Map, Percent } from "lucide-react";
import type { VacuumStatus } from "./contracts";
import roborockImg from "./assets/roborock.png";

interface Props {
  vacuum: VacuumStatus;
}

const STATUS_COLOR: Record<string, string> = {
  charging:  "#60a5fa",
  docked:    "#60a5fa",
  cleaning:  "#4ade80",
  returning: "#fb923c",
  idle:      "#fbbf24",
  error:     "#ef4444",
  paused:    "#a78bfa",
};

function capitalize(s: string) {
  return s ? s.charAt(0).toUpperCase() + s.slice(1) : "Unknown";
}

export function VacuumPopover({ vacuum }: Props) {
  const statusColor = STATUS_COLOR[vacuum.status] ?? "rgba(255,255,255,0.5)";
  const battColor = vacuum.battery < 20 ? "#ef4444" : vacuum.charging ? "#60a5fa" : "#4ade80";
  const BatIcon = vacuum.charging ? BatteryCharging : Battery;
  const isActive = vacuum.cleaning;

  return (
    <div className="vacuum-popover">
      <div className="vacuum-popover-hero">
        <img src={roborockImg} alt="Roborock S7 MaxV" className="vacuum-popover-img" />
        <div className="vacuum-popover-img-fade" />
        <div
          className={`vacuum-popover-state-badge ${isActive ? "vacuum-badge-pulse" : ""}`}
          style={{ color: statusColor, borderColor: statusColor }}
        >
          <span className="vacuum-state-dot" style={{ background: statusColor, boxShadow: `0 0 6px ${statusColor}` }} />
          {capitalize(vacuum.status)}
        </div>
      </div>

      <div className="vacuum-popover-body">
        <div className="vacuum-stat-row">
          <BatIcon size={13} style={{ color: battColor, flexShrink: 0 }} />
          <span className="vacuum-stat-label">Battery</span>
          <div className="vacuum-stat-bar-track">
            <div className="vacuum-stat-bar-fill" style={{ width: `${vacuum.battery}%`, background: battColor }} />
          </div>
          <span className="vacuum-stat-val">{vacuum.battery}%</span>
        </div>

        {isActive && (
          <div className="vacuum-stat-row">
            <Percent size={13} style={{ color: "#4ade80", flexShrink: 0 }} />
            <span className="vacuum-stat-label">Progress</span>
            <div className="vacuum-stat-bar-track">
              <div className="vacuum-stat-bar-fill" style={{ width: `${vacuum.cleaning_progress}%`, background: "#4ade80" }} />
            </div>
            <span className="vacuum-stat-val">{vacuum.cleaning_progress}%</span>
          </div>
        )}

        {vacuum.selected_map && (
          <div className="vacuum-stat-row">
            <Map size={13} style={{ color: "rgba(255,255,255,0.35)", flexShrink: 0 }} />
            <span className="vacuum-stat-label">{vacuum.selected_map}</span>
          </div>
        )}
      </div>
    </div>
  );
}
