import type { PhoneStatus } from "./contracts";
import { DeviceCard, KV, PBar, Mono } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import pixel9Img from "./assets/pixel9.png";

function formatChargeTime(min: number): string {
  if (min < 0) return "";
  if (min < 60) return `${min}m`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function formatActivity(a: string): string {
  return a.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function PhonePopover({ phone }: { phone: PhoneStatus }) {
  const battHue = phone.charging ? HUE.amber
    : phone.battery <= 20 ? HUE.red
    : HUE.phone;
  const chargeTime = formatChargeTime(phone.charge_time_min);
  const hasWifi = phone.wifi_ssid && phone.wifi_ssid !== "unavailable" && phone.wifi_ssid !== "unknown";
  const hasActivity = phone.activity && phone.activity !== "unavailable" && phone.activity !== "unknown";

  return (
    <DeviceCard
      w={248}
      imgSrc={pixel9Img}
      imgH={150}
      title="Phone"
      hue={HUE.phone}
      pill={phone.at_home ? "Home" : "Away"}
    >
      <KV icon={<Icon name={phone.charging ? "bolt" : "battery"} size={12} />} label="Battery" hue={battHue}>
        <PBar pct={phone.battery} hue={battHue} style={{ flex: "none", width: 64 }} />
        <Mono size={11} w={600} style={{ marginLeft: 8, minWidth: 28, textAlign: "right" }}>{phone.battery}%</Mono>
      </KV>
      {phone.charging && chargeTime && (
        <KV icon={<Icon name="clock" size={12} />} label="Full in" hue={HUE.amber}>
          <Mono size={11} color={T.t2}>{chargeTime}</Mono>
        </KV>
      )}
      {hasWifi && (
        <KV icon={<Icon name="wifi" size={12} />} label={phone.wifi_ssid} hue={HUE.phone} />
      )}
      {hasActivity && (
        <KV icon={<Icon name="pin" size={12} />} label={formatActivity(phone.activity)} hue={T.t3} />
      )}
    </DeviceCard>
  );
}
