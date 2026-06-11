import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppSettings,
  CalendarEvent,
  LimitsSnapshot,
  MediaInfo,
  MowerStatus,
  PhoneStatus,
  ProjectsSnapshot,
  VacuumStatus,
  WeatherDetailed,
} from "../contracts";

export interface BluetoothStatus {
  connected: boolean;
  devices: string[];
}

export interface BatteryStatus {
  percentage: number;
  is_charging: boolean;
  has_battery: boolean;
}

export interface ObsStatus {
  is_streaming: boolean;
  is_recording: boolean;
}

const formatTime = (date: Date, use24h: boolean) =>
  date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: !use24h,
  });

export function useMenuBarModel() {
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
  const [volume, setVolume] = useState(0.5);
  const [battery, setBattery] = useState<BatteryStatus | null>(null);
  const [bluetooth, setBluetooth] = useState<BluetoothStatus>({
    connected: false,
    devices: [],
  });
  const [micMuted, setMicMuted] = useState(false);
  const [privacyMode, setPrivacyMode] = useState(false);
  const [obsStatus, setObsStatus] = useState<ObsStatus | null>(null);
  const [weather, setWeather] = useState<WeatherDetailed | null>(null);
  const [usageLimits, setUsageLimits] = useState<LimitsSnapshot | null>(null);
  const [projects, setProjects] = useState<ProjectsSnapshot | null>(null);
  const [projectsRefreshing, setProjectsRefreshing] = useState(false);
  const [mower, setMower] = useState<MowerStatus | null>(null);
  const [vacuum, setVacuum] = useState<VacuumStatus | null>(null);
  const [phone, setPhone] = useState<PhoneStatus | null>(null);
  const [calendar, setCalendar] = useState<CalendarEvent[] | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [use24h, setUse24h] = useState(true);
  const [time, setTime] = useState(() => formatTime(new Date(), true));

  const fetchMedia = useCallback(async () => {
    setMediaInfo(await invoke<MediaInfo | null>("get_media_info_unified"));
  }, []);

  const fetchStatuses = useCallback(async () => {
    const [nextBattery, nextBluetooth, nextMicMuted, nextPrivacyMode, nextObsStatus] =
      await Promise.all([
        invoke<BatteryStatus>("get_battery_status"),
        invoke<BluetoothStatus>("get_bluetooth_status"),
        invoke<boolean>("get_mic_status"),
        invoke<boolean>("get_privacy_status"),
        invoke<ObsStatus>("get_obs_status"),
      ]);
    setBattery(nextBattery);
    setBluetooth(nextBluetooth);
    setMicMuted(nextMicMuted);
    setPrivacyMode(nextPrivacyMode);
    setObsStatus(nextObsStatus);
  }, []);

  const fetchWeather = useCallback(async (nextSettings?: AppSettings) => {
    const resolvedSettings =
      nextSettings ?? (await invoke<AppSettings>("get_settings"));
    if (
      resolvedSettings.weather_lat === null ||
      resolvedSettings.weather_lon === null
    ) {
      setWeather(null);
      return;
    }
    setWeather(
      await invoke<WeatherDetailed>("get_weather", {
        lat: resolvedSettings.weather_lat,
        lon: resolvedSettings.weather_lon,
        placeName: resolvedSettings.weather_location || "Unknown",
      }),
    );
  }, []);

  const fetchUsageLimits = useCallback(async () => {
    setUsageLimits(await invoke<LimitsSnapshot>("get_usage_limits"));
  }, []);

  const fetchProjects = useCallback(async (refresh = false) => {
    setProjectsRefreshing(true);
    try {
      setProjects(
        await invoke<ProjectsSnapshot | null>("get_projects", { refresh }),
      );
    } finally {
      setProjectsRefreshing(false);
    }
  }, []);

  const fetchMower = useCallback(async () => {
    setMower(await invoke<MowerStatus | null>("get_mower_status"));
  }, []);

  const fetchVacuum = useCallback(async () => {
    setVacuum(await invoke<VacuumStatus | null>("get_ha_vacuum_status"));
  }, []);

  const fetchPhone = useCallback(async () => {
    setPhone(await invoke<PhoneStatus | null>("get_ha_phone_status"));
  }, []);

  const fetchCalendar = useCallback(async () => {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).toISOString();
    const end = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 7).toISOString();
    setCalendar(await invoke<CalendarEvent[] | null>("get_calendar_events", { start, end }));
  }, []);

  useEffect(() => {
    let disposed = false;
    const report = (operation: string, error: unknown) =>
      console.error(`[menu-bar] ${operation} failed`, error);

    invoke<AppSettings>("get_settings")
      .then((nextSettings) => {
        if (disposed) return;
        setSettings(nextSettings);
        setUse24h(nextSettings.use_24h !== false);
        document.documentElement.style.setProperty(
          "--accent",
          nextSettings.accent_color,
        );
      })
      .catch((error) => report("load settings", error));
    invoke<number>("get_volume")
      .then(setVolume)
      .catch((error) => report("load volume", error));
    fetchMedia().catch((error) => report("load media", error));
    fetchStatuses().catch((error) => report("load statuses", error));
    fetchWeather().catch((error) => report("load weather", error));
    fetchUsageLimits().catch((error) => report("load usage limits", error));
    fetchProjects().catch((error) => report("load projects", error));
    fetchMower().catch((error) => report("load mower", error));
    fetchVacuum().catch((error) => report("load vacuum", error));
    fetchPhone().catch((error) => report("load phone", error));
    fetchCalendar().catch((error) => report("load calendar", error));

    const statusInterval = window.setInterval(
      () => fetchStatuses().catch((error) => report("refresh statuses", error)),
      5000,
    );
    const weatherInterval = window.setInterval(
      () => fetchWeather().catch((error) => report("refresh weather", error)),
      600000,
    );
    const usageLimitsInterval = window.setInterval(
      () =>
        fetchUsageLimits().catch((error) =>
          report("refresh usage limits", error),
        ),
      60000,
    );
    const projectsInterval = window.setInterval(
      () => fetchProjects().catch((error) => report("refresh projects", error)),
      300000,
    );
    const mowerInterval = window.setInterval(
      () => fetchMower().catch((error) => report("refresh mower", error)),
      60000,
    );
    const vacuumInterval = window.setInterval(
      () => fetchVacuum().catch((error) => report("refresh vacuum", error)),
      30000,
    );
    const phoneInterval = window.setInterval(
      () => fetchPhone().catch((error) => report("refresh phone", error)),
      60000,
    );
    const calendarInterval = window.setInterval(
      () => fetchCalendar().catch((error) => report("refresh calendar", error)),
      300000,
    );
    const unlisteners = [
      listen<MediaInfo | null>("media-change", ({ payload }) =>
        setMediaInfo(payload),
      ),
      listen<AppSettings>("settings-changed", ({ payload }) => {
        setSettings(payload);
        setUse24h(payload.use_24h !== false);
        document.documentElement.style.setProperty("--accent", payload.accent_color);
        fetchWeather(payload).catch((error) =>
          report("refresh weather after settings change", error),
        );
        fetchMedia().catch((error) =>
          report("refresh media after settings change", error),
        );
        fetchProjects(true).catch((error) =>
          report("refresh projects after settings change", error),
        );
      }),
    ];

    return () => {
      disposed = true;
      window.clearInterval(statusInterval);
      window.clearInterval(weatherInterval);
      window.clearInterval(usageLimitsInterval);
      window.clearInterval(projectsInterval);
      window.clearInterval(mowerInterval);
      window.clearInterval(vacuumInterval);
      window.clearInterval(phoneInterval);
      window.clearInterval(calendarInterval);
      Promise.all(unlisteners).then((callbacks) =>
        callbacks.forEach((unlisten) => unlisten()),
      );
    };
  }, [fetchCalendar, fetchMedia, fetchMower, fetchPhone, fetchProjects, fetchStatuses, fetchUsageLimits, fetchVacuum, fetchWeather]);

  useEffect(() => {
    setTime(formatTime(new Date(), use24h));
    const timer = window.setInterval(
      () => setTime(formatTime(new Date(), use24h)),
      10000,
    );
    return () => window.clearInterval(timer);
  }, [use24h]);

  const controlMedia = async (action: "previous" | "play_pause" | "next") => {
    await invoke("media_control_unified", { action });
  };

  const changeVolume = (nextVolume: number) => {
    setVolume(nextVolume);
    return invoke("set_volume", { volume: nextVolume });
  };

  const toggleMic = async () => {
    setMicMuted(await invoke<boolean>("toggle_mic_mute"));
  };

  const togglePrivacy = async () => {
    const enabled = !privacyMode;
    await invoke("set_privacy_mode", { enabled });
    setPrivacyMode(enabled);
    setMicMuted(enabled || (await invoke<boolean>("get_mic_status")));
  };

  return {
    battery,
    bluetooth,
    calendar,
    changeVolume,
    controlMedia,
    mediaInfo,
    micMuted,
    mower,
    phone,
    vacuum,
    obsStatus,
    privacyMode,
    projects,
    projectsRefreshing,
    settings,
    time,
    toggleMic,
    togglePrivacy,
    refreshProjects: () => fetchProjects(true),
    usageLimits,
    volume,
    weather,
  };
}
