import { BatteryCharging, Battery, Wifi, MapPin, Activity, Clock } from "lucide-react";
import type { PhoneStatus } from "./contracts";
import pixel9Img from "./assets/pixel9.png";

interface Props {
  phone: PhoneStatus;
}

function battColor(level: number, charging: boolean): string {
  if (charging) return "#4ade80";
  if (level <= 20) return "#ef4444";
  if (level <= 40) return "#fb923c";
  return "#60a5fa";
}

function formatChargeTime(min: number): string {
  if (min < 0) return "";
  if (min < 60) return `${min}m`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function formatActivity(a: string): string {
  return a.replace(/_/g, " ").replace(/\b\w/g, c => c.toUpperCase());
}

export function PhonePopover({ phone }: Props) {
  const color = battColor(phone.battery, phone.charging);
  const BatIcon = phone.charging ? BatteryCharging : Battery;
  const chargeTime = formatChargeTime(phone.charge_time_min);
  const presenceColor = phone.at_home ? "#4ade80" : "#fb923c";
  const hasWifi = phone.wifi_ssid && phone.wifi_ssid !== "unavailable" && phone.wifi_ssid !== "unknown";
  const hasActivity = phone.activity && phone.activity !== "unavailable" && phone.activity !== "unknown";

  return (
    <div className="phone-popover">
      <div className="phone-popover-hero">
        <img src={pixel9Img} alt="Pixel 9 Pro XL" className="phone-popover-img" />
        <div className="phone-popover-img-fade" />
        <div
          className="phone-popover-state-badge"
          style={{ color: presenceColor, borderColor: presenceColor }}
        >
          <MapPin size={10} style={{ flexShrink: 0 }} />
          {phone.at_home ? "Home" : "Away"}
        </div>
      </div>

      <div className="phone-popover-body">
        <div className="phone-stat-row">
          <BatIcon size={13} style={{ color, flexShrink: 0 }} />
          <span className="phone-stat-label">Battery</span>
          <div className="phone-stat-bar-track">
            <div className="phone-stat-bar-fill" style={{ width: `${phone.battery}%`, background: color }} />
          </div>
          <span className="phone-stat-val">{phone.battery}%</span>
        </div>

        {phone.charging && chargeTime && (
          <div className="phone-stat-row">
            <Clock size={13} style={{ color: "#a78bfa", flexShrink: 0 }} />
            <span className="phone-stat-label">Full in</span>
            <span className="phone-stat-val" style={{ color: "#a78bfa", marginLeft: "auto" }}>{chargeTime}</span>
          </div>
        )}

        {hasWifi && (
          <div className="phone-stat-row">
            <Wifi size={13} style={{ color: "#60a5fa", flexShrink: 0 }} />
            <span className="phone-stat-label" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {phone.wifi_ssid}
            </span>
          </div>
        )}

        {hasActivity && (
          <div className="phone-stat-row">
            <Activity size={13} style={{ color: "rgba(255,255,255,0.35)", flexShrink: 0 }} />
            <span className="phone-stat-label">{formatActivity(phone.activity)}</span>
          </div>
        )}
      </div>
    </div>
  );
}
