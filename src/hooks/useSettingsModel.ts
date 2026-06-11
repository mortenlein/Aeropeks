import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  LocationResult,
  ModulesConfig,
  TerminalShortcut,
} from "../contracts";
import { defaultModules } from "../contracts";

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
  const [usageHiddenProviders, setUsageHiddenProviders] = useState<string[]>([]);
  const [reserveScreenSpace, setReserveScreenSpace] = useState(true);
  const [hideNativeTaskbar, setHideNativeTaskbar] = useState(false);
  const [shellMessage, setShellMessage] = useState("");
  const [haUrl, setHaUrl] = useState("");
  const [haToken, setHaToken] = useState("");
  const [haPollSeconds, setHaPollSeconds] = useState(30);
  const [modules, setModules] = useState<ModulesConfig>(defaultModules());

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
        setUsageHiddenProviders(settings.usage_hidden_providers ?? []);
        setUse24h(settings.use_24h);
        setReserveScreenSpace(settings.reserve_screen_space);
        setHideNativeTaskbar(settings.hide_native_taskbar);
        setHaUrl(settings.homeassistant_url);
        setHaToken(settings.homeassistant_token);
        setHaPollSeconds(settings.homeassistant_poll_seconds ?? 30);
        setModules(settings.modules ?? defaultModules());
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
          usage_hidden_providers: usageHiddenProviders,
          use_24h: use24h,
          reserve_screen_space: reserveScreenSpace,
          hide_native_taskbar: hideNativeTaskbar,
          homeassistant_url: haUrl,
          homeassistant_token: haToken,
          homeassistant_poll_seconds: haPollSeconds,
          modules,
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

  const handleRestoreShell = async () => {
    try {
      await invoke("restore_shell_state");
      setShellMessage("Windows taskbar and work area restored.");
    } catch (error) {
      setShellMessage(`Restore failed: ${String(error)}`);
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

  const updateModule = <K extends keyof ModulesConfig>(
    id: K,
    patch: Partial<ModulesConfig[K]>,
  ) =>
    setModules((current) => ({
      ...current,
      [id]: { ...current[id], ...patch },
    }));

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
    usageHiddenProviders,
    setUsageHiddenProviders,
    removeShortcut,
    reserveScreenSpace,
    saved,
    searchQuery,
    searchResults,
    selectLocation,
    setAccentColor,
    setHaUrl,
    setHaToken,
    haPollSeconds,
    setHaPollSeconds,
    modules,
    updateModule,
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
