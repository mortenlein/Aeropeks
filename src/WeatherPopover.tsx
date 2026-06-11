import type { WeatherDetailed } from "./contracts";
import { Panel, Card, Mono, Micro } from "./atoms";
import { Icon } from "./icons";
import { HUE, T } from "./tokens";
import type { ReactNode } from "react";

function toSentenceCase(s: string): string {
  const clean = s.toLowerCase().replace(/_/g, " ");
  return clean.charAt(0).toUpperCase() + clean.slice(1);
}

function getWeatherIcon(symbol: string, size = 14, colorOverride?: string): ReactNode {
  const s = symbol.toLowerCase();
  const c = colorOverride;
  if (s.includes("clearsky") || s.includes("fair")) return <Icon name="sun" size={size} style={{ color: c ?? HUE.weather }} />;
  if (s.includes("partlycloudy")) return <Icon name="partly" size={size} style={{ color: c ?? HUE.weather }} />;
  if (s.includes("cloudy") || s.includes("snow") || s.includes("fog") || s.includes("mist")) return <Icon name="cloud" size={size} style={{ color: c ?? T.t3 }} />;
  if (s.includes("rain") || s.includes("drizzle") || s.includes("sleet")) return <Icon name="rain" size={size} style={{ color: c ?? HUE.weather }} />;
  if (s.includes("thunder")) return <Icon name="rain" size={size} style={{ color: c ?? HUE.amber }} />;
  if (s.includes("night")) return <Icon name="moon" size={size} style={{ color: c ?? T.t3 }} />;
  return <Icon name="cloud" size={size} style={{ color: c ?? T.t3 }} />;
}

const formatTime = (iso: string) => {
  const d = new Date(iso);
  return d.getHours().toString().padStart(2, "0") + ":00";
};

const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString("nb-NO", { weekday: "short", day: "numeric" });

interface Props {
  data: WeatherDetailed;
  onClose: () => void;
}

export function WeatherPopover({ data, onClose }: Props) {
  return (
    <Panel
      w={380}
      title="Weather"
      icon={<Icon name="cloud" size={13} />}
      hue={HUE.weather}
      style={{ right: 0 }}
      onClose={onClose}
    >
      {/* Hero: big temp + location beside it */}
      <div style={{ display: "flex", alignItems: "flex-start", gap: 14, marginBottom: 18 }}>
        <span style={{ fontSize: 46, fontWeight: 600, lineHeight: 1, letterSpacing: "-0.03em", color: T.t1 }}>
          {Math.round(data.temp)}°
        </span>
        <div style={{ paddingTop: 4 }}>
          <div style={{ fontSize: 14.5, fontWeight: 600, color: T.t1 }}>{data.place_name}</div>
          <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 4 }}>
            <span style={{ color: HUE.weather, display: "flex" }}>{getWeatherIcon(data.symbol, 12)}</span>
            <span style={{ fontSize: 11.5, color: T.t2 }}>{toSentenceCase(data.symbol)}</span>
          </div>
        </div>
      </div>

      {/* Hourly — single Card wrapping 6 fixed columns */}
      <Micro color={HUE.weather} style={{ marginBottom: 10 }}>Hourly</Micro>
      <Card pad={10} style={{ display: "flex", marginBottom: 16 }}>
        {data.hourly.slice(0, 6).map((h, i) => (
          <div key={i} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", gap: 7 }}>
            <Mono size={9} color={T.t3}>{i === 0 ? "Now" : formatTime(h.time)}</Mono>
            <span style={{ display: "flex", color: HUE.weather }}>{getWeatherIcon(h.symbol, 13)}</span>
            <Mono size={11.5} w={600}>{Math.round(h.temp)}°</Mono>
          </div>
        ))}
      </Card>

      {/* Daily — plain rows with borderTop dividers, no individual card bg */}
      <Micro color={HUE.weather} style={{ marginBottom: 4 }}>Next 7 days</Micro>
      <div style={{ display: "flex", flexDirection: "column" }}>
        {data.daily.map((d, i) => (
          <div
            key={i}
            style={{ display: "flex", alignItems: "center", gap: 10, minHeight: 34, borderTop: i > 0 ? `1px solid ${T.divider}` : "none" }}
          >
            <span style={{ fontFamily: 'var(--font-ui)', fontSize: 12, color: i === 0 ? T.t1 : T.t2, width: 64, fontWeight: i === 0 ? 600 : 400, flexShrink: 0, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {i === 0 ? "Today" : formatDate(d.date)}
            </span>
            <span style={{ display: "flex", flexShrink: 0 }}>{getWeatherIcon(d.symbol, 13, T.t3)}</span>
            <span style={{ flex: 1 }} />
            <Mono size={11.5} w={600} style={{ width: 28, textAlign: "right" }}>{Math.round(d.temp_max)}°</Mono>
            <Mono size={11.5} color={T.t3} style={{ width: 28, textAlign: "right" }}>{Math.round(d.temp_min)}°</Mono>
            <Mono size={10} color={T.t3} style={{ width: 36, textAlign: "right" }}>{Math.round(d.humidity)}%</Mono>
          </div>
        ))}
      </div>
    </Panel>
  );
}
