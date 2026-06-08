import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WeatherPopover } from "./WeatherPopover";
import { ProjectsPopover } from "./ProjectsPopover";
import { UsageLimitsPopover } from "./UsageLimitsPopover";
import type {
  AppSettings,
  LimitsSnapshot,
  ProjectsSnapshot,
  WeatherDetailed,
} from "./contracts";

function DemoWrapper({ children }: { children: React.ReactNode }) {
  return (
    <div
      className="demo-popup-outer"
      onContextMenu={(e) => e.preventDefault()}
    >
      <div className="demo-drag-handle" data-tauri-drag-region>
        <button
          className="demo-exit-btn"
          onClick={() => invoke("exit_demo_mode").catch(console.error)}
          title="Exit Screenshot Mode"
        >
          ✕ exit screenshot mode
        </button>
      </div>
      <div className="demo-popup-content">{children}</div>
    </div>
  );
}

function Loading() {
  return <div className="demo-loading">Loading…</div>;
}

export function DemoWeather() {
  const [data, setData] = useState<WeatherDetailed | null>(null);

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then((s) => {
        if (s.weather_lat == null || s.weather_lon == null) return;
        return invoke<WeatherDetailed>("get_weather", {
          lat: s.weather_lat,
          lon: s.weather_lon,
          placeName: s.weather_location || "Unknown",
        });
      })
      .then((w) => w && setData(w))
      .catch(console.error);
  }, []);

  return (
    <DemoWrapper>
      {data ? <WeatherPopover data={data} onClose={() => {}} /> : <Loading />}
    </DemoWrapper>
  );
}

export function DemoUsage() {
  const [data, setData] = useState<LimitsSnapshot | null>(null);

  useEffect(() => {
    invoke<LimitsSnapshot>("get_usage_limits").then(setData).catch(console.error);
  }, []);

  return (
    <DemoWrapper>
      {data ? <UsageLimitsPopover snapshot={data} /> : <Loading />}
    </DemoWrapper>
  );
}

export function DemoProjects() {
  const [data, setData] = useState<ProjectsSnapshot | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    invoke<ProjectsSnapshot | null>("get_projects")
      .then((d) => d && setData(d))
      .catch(console.error);
  }, []);

  const onRefresh = () => {
    setRefreshing(true);
    invoke<ProjectsSnapshot | null>("get_projects", { refresh: true })
      .then((d) => d && setData(d))
      .catch(console.error)
      .finally(() => setRefreshing(false));
  };

  return (
    <DemoWrapper>
      {data ? (
        <ProjectsPopover
          snapshot={data}
          refreshing={refreshing}
          onRefresh={onRefresh}
        />
      ) : (
        <Loading />
      )}
    </DemoWrapper>
  );
}
