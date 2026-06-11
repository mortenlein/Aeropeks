import type { MowerStatus } from "./contracts";
import { DeviceCard, Card, KV, Stat, Mono } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import mowerImg from "./assets/mower.webp";

function formatHours(minutes: number): string {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function formatArea(m2: number): string {
  return m2 >= 10000 ? `${(m2 / 10000).toFixed(2)} ha` : `${m2} m²`;
}

export function MowerPopover({ mower }: { mower: MowerStatus }) {
  const online = mower.state !== "unavailable" && mower.state !== "";
  const pulse = mower.state === "mowing";

  return (
    <DeviceCard
      w={264}
      imgSrc={mowerImg}
      title="Mower"
      hue="var(--hue-mower)"
      pill={mower.state_label}
      pillPulse={pulse}
      footer={
        <KV icon={<Icon name="chip" size={11} />} label="Firmware">
          <Mono size={10.5} color={T.t3}>fw {mower.firmware}</Mono>
        </KV>
      }
    >
      <KV
        icon={<Icon name="wifi" size={12} />}
        label={online ? "Online" : "Offline"}
        hue={online ? HUE.ok : HUE.red}
      />
      {(mower.dnd || mower.has_update) && (
        <KV icon={<span style={{ width: 12 }} />} label="Notices">
          <span style={{ display: "flex", gap: 8 }}>
            {mower.dnd && <Mono size={9.5} color={HUE.mower}>DnD</Mono>}
            {mower.has_update && <Mono size={9.5} color={HUE.amber}>Update</Mono>}
          </span>
        </KV>
      )}
      {mower.zone_state && mower.zone_state !== "unknown" && mower.zone_state !== "" && (
        <KV icon={<Icon name="map" size={12} />} label={`Zone ${mower.zone_id} — ${mower.zone_state}`} hue={T.t3} />
      )}
      <Card style={{ display: "flex", marginTop: 10 }} pad={0}>
        <Stat value={String(mower.cleaning_count)} label="sessions" />
        <Stat value={formatArea(mower.total_area_m2)} label="total area" />
        <Stat value={formatHours(mower.total_time_min)} label="runtime" />
      </Card>
    </DeviceCard>
  );
}
