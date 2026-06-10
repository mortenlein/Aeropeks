import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  DebugWindowInfo,
  LocationResult,
  TerminalShortcut,
} from "../contracts";

export function useSettingsModel() {
  const [plexUrl, setPlexUrl] = useState("");
  const [plexToken, setPlexToken] = useState("");
  const [accentColor, setAccentColor] = useState("#22c55e");
  const [shortcuts, setShortcuts] = useState<TerminalShortcut[]>([]);
  const [saved, setSaved] = useState(false);
  const [use24h, setUse24h] = useState(true);
  const [weatherLocation, setWeatherLocation] = useState("");
  const [weatherLat, setWeatherLat] = useState<number | null>(null);
  const [weatherLon, setWeatherLon] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<LocationResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [obsUrl, setObsUrl] = useState("");
  const [obsPassword, setObsPassword] = useState("");
  const [githubToken, setGithubToken] = useState("");
  const [usageLimitsUrl, setUsageLimitsUrl] = useState("");
  const [reserveScreenSpace, setReserveScreenSpace] = useState(true);
  const [hideNativeTaskbar, setHideNativeTaskbar] = useState(false);
  const [debugInspector, setDebugInspector] = useState(false);
  const [debugWindows, setDebugWindows] = useState<DebugWindowInfo[]>([]);
  const [shellMessage, setShellMessage] = useState("");
  const [dreameUsername, setDreameUsername] = useState("");
  const [dreamePassword, setDreamePassword] = useState("");
  const [dreameDeviceId, setDreameDeviceId] = useState("");
  const [haUrl, setHaUrl] = useState("");
  const [haToken, setHaToken] = useState("");
  const [haCalendarEntityId, setHaCalendarEntityId] = useState("");

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then((settings) => {
        setPlexUrl(settings.plex_url);
        setPlexToken(settings.plex_token);
        setAccentColor(settings.accent_color);
        setShortcuts(settings.terminal_shortcuts);
        setWeatherLocation(settings.weather_location);
        setWeatherLat(settings.weather_lat);
        setWeatherLon(settings.weather_lon);
        setSearchQuery(settings.weather_location);
        setObsUrl(settings.obs_websocket_url);
        setObsPassword(settings.obs_websocket_password);
        setGithubToken(settings.github_token);
        setUsageLimitsUrl(settings.usage_limits_url);
        setUse24h(settings.use_24h);
        setReserveScreenSpace(settings.reserve_screen_space);
        setHideNativeTaskbar(settings.hide_native_taskbar);
        setDebugInspector(settings.debug_inspector);
        setDreameUsername(settings.dreame_username);
        setDreamePassword(settings.dreame_password);
        setDreameDeviceId(settings.dreame_device_id);
        setHaUrl(settings.homeassistant_url);
        setHaToken(settings.homeassistant_token);
        setHaCalendarEntityId(settings.ha_calendar_entity_id);
      })
      .catch((error) => setShellMessage(`Settings load failed: ${String(error)}`));
  }, []);

  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (query.length < 3) {
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    try {
      setSearchResults(
        await invoke<LocationResult[]>("search_locations", { query }),
      );
    } catch (error) {
      setSearchResults([]);
      setShellMessage(`Location search failed: ${String(error)}`);
    } finally {
      setIsSearching(false);
    }
  };

  const selectLocation = (location: LocationResult) => {
    setWeatherLocation(location.name);
    setWeatherLat(location.lat);
    setWeatherLon(location.lon);
    setSearchQuery(location.name);
    setSearchResults([]);
  };

  const handleSave = async () => {
    try {
      await invoke("save_settings", {
        settings: {
          plex_url: plexUrl,
          plex_token: plexToken,
          accent_color: accentColor,
          terminal_shortcuts: shortcuts,
          weather_location: weatherLocation,
          weather_lat: weatherLat,
          weather_lon: weatherLon,
          obs_websocket_url: obsUrl,
          obs_websocket_password: obsPassword,
          github_token: githubToken,
          usage_limits_url: usageLimitsUrl,
          use_24h: use24h,
          reserve_screen_space: reserveScreenSpace,
          hide_native_taskbar: hideNativeTaskbar,
          debug_inspector: debugInspector,
          dreame_username: dreameUsername,
          dreame_password: dreamePassword,
          dreame_device_id: dreameDeviceId,
          homeassistant_url: haUrl,
          homeassistant_token: haToken,
          ha_calendar_entity_id: haCalendarEntityId,
        },
      });
      await invoke("register_hotkeys");
      document.documentElement.style.setProperty("--accent", accentColor);
      setShellMessage("");
      setSaved(true);
      window.setTimeout(() => setSaved(false), 2000);
    } catch (error) {
      setShellMessage(`Settings save failed: ${String(error)}`);
    }
  };

  const refreshDebugWindows = async () => {
    try {
      setDebugWindows(
        await invoke<DebugWindowInfo[]>("get_window_debug_snapshot"),
      );
    } catch (error) {
      setShellMessage(`Window snapshot failed: ${String(error)}`);
    }
  };

  const handleRestoreShell = async () => {
    try {
      await invoke("restore_shell_state");
      setShellMessage("Windows taskbar and work area restored.");
    } catch (error) {
      setShellMessage(`Restore failed: ${String(error)}`);
    }
  };

  const handleClearIconCache = async () => {
    try {
      await invoke("clear_icon_cache");
      await refreshDebugWindows();
      setShellMessage("Icon cache cleared. Aeropeks will rebuild it on refresh.");
    } catch (error) {
      setShellMessage(`Icon cache clear failed: ${String(error)}`);
    }
  };

  const addShortcut = () =>
    setShortcuts((current) => [
      ...current,
      {
        id: `ssh-${Date.now()}`,
        label: "New Shortcut",
        cmd: "echo Hello",
        shortcut: "Alt+Shift+T",
      },
    ]);

  const removeShortcut = (id: string) =>
    setShortcuts((current) => current.filter((shortcut) => shortcut.id !== id));

  const updateShortcut = (
    id: string,
    field: keyof TerminalShortcut,
    value: string,
  ) =>
    setShortcuts((current) =>
      current.map((shortcut) =>
        shortcut.id === id ? { ...shortcut, [field]: value } : shortcut,
      ),
    );

  return {
    accentColor,
    addShortcut,
    debugInspector,
    debugWindows,
    dreameDeviceId,
    dreamePassword,
    dreameUsername,
    handleClearIconCache,
    handleRestoreShell,
    handleSave,
    handleSearch,
    githubToken,
    hideNativeTaskbar,
    isSearching,
    obsPassword,
    obsUrl,
    plexToken,
    plexUrl,
    usageLimitsUrl,
    setUsageLimitsUrl,
    refreshDebugWindows,
    removeShortcut,
    reserveScreenSpace,
    saved,
    searchQuery,
    searchResults,
    selectLocation,
    setAccentColor,
    setDebugInspector,
    setDreameDeviceId,
    setDreamePassword,
    setDreameUsername,
    setHaUrl,
    setHaToken,
    haCalendarEntityId,
    setHaCalendarEntityId,
    setHideNativeTaskbar,
    setGithubToken,
    setObsPassword,
    setObsUrl,
    setPlexToken,
    setPlexUrl,
    setReserveScreenSpace,
    setUse24h,
    shellMessage,
    shortcuts,
    updateShortcut,
    use24h,
    haUrl,
    haToken,
    weatherLat,
    weatherLocation,
    weatherLon,
  };
}
