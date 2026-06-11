import type { VacuumStatus } from "./contracts";
import { DeviceCard, KV, PBar, Mono } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import roborockImg from "./assets/roborock.png";

function capitalize(s: string) {
  return s ? s.charAt(0).toUpperCase() + s.slice(1) : "Unknown";
}

export function VacuumPopover({ vacuum }: { vacuum: VacuumStatus }) {
  const pulse = vacuum.cleaning;
  const battHue = vacuum.battery < 20 ? HUE.red : HUE.vacuum;
  return (
    <DeviceCard
      w={264}
      imgSrc={roborockImg}
      title="Vacuum"
      hue={HUE.vacuum}
      pill={capitalize(vacuum.status)}
      pillPulse={pulse}
    >
      <KV icon={<Icon name={vacuum.charging ? "bolt" : "battery"} size={12} />} label="Battery" hue={battHue}>
        <PBar pct={vacuum.battery} hue={battHue} style={{ flex: "none", width: 72 }} />
        <Mono size={11} w={600} style={{ marginLeft: 8, minWidth: 28, textAlign: "right" }}>{vacuum.battery}%</Mono>
      </KV>
      {vacuum.cleaning && (
        <KV icon={<Icon name="refresh" size={12} />} label="Progress" hue={HUE.ok}>
          <PBar pct={vacuum.cleaning_progress} hue={HUE.ok} style={{ flex: "none", width: 72 }} />
          <Mono size={11} w={600} style={{ marginLeft: 8, minWidth: 28, textAlign: "right" }}>{vacuum.cleaning_progress}%</Mono>
        </KV>
      )}
      {vacuum.selected_map && (
        <KV icon={<Icon name="map" size={12} />} label="Map">
          <Mono size={11} color={T.t2}>{vacuum.selected_map}</Mono>
        </KV>
      )}
    </DeviceCard>
  );
}
