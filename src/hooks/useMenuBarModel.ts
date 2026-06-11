import { useCallback, useEffect, useRef, useState } from "react";
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

const report = (operation: string, error: unknown) =>
  console.error(`[menu-bar] ${operation} failed`, error);

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

  // The media-change event arrives regardless of module state; this ref lets
  // the listener (bound once) respect the current toggle.
  const mediaEnabledRef = useRef(true);

  const applySettings = useCallback((next: AppSettings) => {
    setSettings(next);
    setUse24h(next.use_24h !== false);
    document.documentElement.style.setProperty("--accent", next.accent_color);
  }, []);

  const fetchMedia = useCallback(async () => {
    setMediaInfo(await invoke<MediaInfo | null>("get_media_info_unified"));
  }, []);

  const fetchCoreStatuses = useCallback(async () => {
    const [nextBattery, nextBluetooth, nextMicMuted, nextPrivacyMode] =
      await Promise.all([
        invoke<BatteryStatus>("get_battery_status"),
        invoke<BluetoothStatus>("get_bluetooth_status"),
        invoke<boolean>("get_mic_status"),
        invoke<boolean>("get_privacy_status"),
      ]);
    setBattery(nextBattery);
    setBluetooth(nextBluetooth);
    setMicMuted(nextMicMuted);
    setPrivacyMode(nextPrivacyMode);
  }, []);

  const fetchObs = useCallback(async () => {
    setObsStatus(await invoke<ObsStatus>("get_obs_status"));
  }, []);

  const fetchWeather = useCallback(async (forSettings: AppSettings) => {
    if (forSettings.weather_lat === null || forSettings.weather_lon === null) {
      setWeather(null);
      return;
    }
    setWeather(
      await invoke<WeatherDetailed>("get_weather", {
        lat: forSettings.weather_lat,
        lon: forSettings.weather_lon,
        placeName: forSettings.weather_location || "Unknown",
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

  // Core: settings, volume, system statuses, event listeners. Runs once.
  useEffect(() => {
    let disposed = false;

    invoke<AppSettings>("get_settings")
      .then((next) => {
        if (!disposed) applySettings(next);
      })
      .catch((error) => report("load settings", error));
    invoke<number>("get_volume")
      .then(setVolume)
      .catch((error) => report("load volume", error));
    fetchCoreStatuses().catch((error) => report("load statuses", error));

    const statusInterval = window.setInterval(
      () => fetchCoreStatuses().catch((error) => report("refresh statuses", error)),
      5000,
    );
    const unlisteners = [
      listen<MediaInfo | null>("media-change", ({ payload }) => {
        if (mediaEnabledRef.current) setMediaInfo(payload);
      }),
      listen<AppSettings>("settings-changed", ({ payload }) => {
        // Module polling reconfigures via the effect below.
        applySettings(payload);
      }),
    ];

    return () => {
      disposed = true;
      window.clearInterval(statusInterval);
      Promise.all(unlisteners).then((callbacks) =>
        callbacks.forEach((unlisten) => unlisten()),
      );
    };
  }, [applySettings, fetchCoreStatuses]);

  // Modules: fetch + poll only what is enabled and configured.
  // Re-runs whenever settings change, tearing down disabled modules.
  useEffect(() => {
    if (!settings) return;
    const m = settings.modules;
    const timers: number[] = [];
    const poll = (fetch: () => Promise<void>, ms: number, label: string) => {
      fetch().catch((error) => report(`load ${label}`, error));
      timers.push(
        window.setInterval(
          () => fetch().catch((error) => report(`refresh ${label}`, error)),
          ms,
        ),
      );
    };

    mediaEnabledRef.current = m.media.enabled;
    if (m.media.enabled) {
      // Event-driven afterwards; backend pushes media-change.
      fetchMedia().catch((error) => report("load media", error));
    } else {
      setMediaInfo(null);
    }

    if (m.weather.enabled && settings.weather_lat !== null && settings.weather_lon !== null) {
      poll(() => fetchWeather(settings), 600000, "weather");
    } else {
      setWeather(null);
    }

    if (m.obs.enabled && settings.obs_websocket_url.trim() !== "") {
      poll(fetchObs, 5000, "obs");
    } else {
      setObsStatus(null);
    }

    if (m.usage_limits.enabled && settings.usage_limits_url.trim() !== "") {
      poll(fetchUsageLimits, 60000, "usage limits");
    } else {
      setUsageLimits(null);
    }

    if (m.projects.enabled) {
      poll(() => fetchProjects(), 300000, "projects");
    } else {
      setProjects(null);
    }

    const haReady = settings.homeassistant_url.trim() !== "";
    if (haReady && m.mower.enabled && m.mower.entity_id !== "") {
      poll(fetchMower, 60000, "mower");
    } else {
      setMower(null);
    }
    if (haReady && m.vacuum.enabled && m.vacuum.entity_id !== "") {
      poll(fetchVacuum, 30000, "vacuum");
    } else {
      setVacuum(null);
    }
    if (haReady && m.phone.enabled && m.phone.device_slug !== "") {
      poll(fetchPhone, 60000, "phone");
    } else {
      setPhone(null);
    }
    if (haReady && m.calendar.enabled && m.calendar.entity_id !== "") {
      poll(fetchCalendar, 300000, "calendar");
    } else {
      setCalendar(null);
    }

    return () => timers.forEach((timer) => window.clearInterval(timer));
  }, [settings, fetchCalendar, fetchMedia, fetchMower, fetchObs, fetchPhone,
      fetchProjects, fetchUsageLimits, fetchVacuum, fetchWeather]);

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
