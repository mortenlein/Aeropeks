export interface TerminalShortcut {
  id: string;
  label: string;
  cmd: string;
  shortcut: string;
}

export interface SimpleModule {
  enabled: boolean;
}

export interface HaEntityModule {
  enabled: boolean;
  entity_id: string;
}

export interface MowerModule {
  enabled: boolean;
  entity_id: string;
  update_entity_id: string;
}

export interface PhoneModule {
  enabled: boolean;
  device_slug: string;
}

export interface ModulesConfig {
  media: SimpleModule;
  weather: SimpleModule;
  usage_limits: SimpleModule;
  projects: SimpleModule;
  obs: SimpleModule;
  camera: HaEntityModule;
  vacuum: HaEntityModule;
  calendar: HaEntityModule;
  mower: MowerModule;
  phone: PhoneModule;
}

export const defaultModules = (): ModulesConfig => ({
  media: { enabled: true },
  weather: { enabled: true },
  usage_limits: { enabled: true },
  projects: { enabled: true },
  obs: { enabled: true },
  camera: { enabled: true, entity_id: "" },
  vacuum: { enabled: true, entity_id: "" },
  calendar: { enabled: true, entity_id: "" },
  mower: { enabled: true, entity_id: "", update_entity_id: "" },
  phone: { enabled: true, device_slug: "" },
});

export interface AppSettings {
  plex_url: string;
  plex_token: string;
  accent_color: string;
  terminal_shortcuts: TerminalShortcut[];
  weather_location: string;
  weather_lat: number | null;
  weather_lon: number | null;
  obs_websocket_url: string;
  obs_websocket_password: string;
  github_token: string;
  usage_limits_url: string;
  usage_hidden_providers: string[];
  use_24h: boolean;
  reserve_screen_space: boolean;
  hide_native_taskbar: boolean;
  homeassistant_url: string;
  homeassistant_token: string;
  homeassistant_poll_seconds: number;
  modules: ModulesConfig;
}

/** Pushed by the backend HA poller via the "ha-snapshot" event. */
export interface HaSnapshot {
  vacuum: VacuumStatus | null;
  mower: MowerStatus | null;
  phone: PhoneStatus | null;
}

export interface VacuumStatus {
  state: string;
  battery: number;
  charging: boolean;
  cleaning: boolean;
  cleaning_progress: number;
  status: string;
  selected_map: string;
}

export interface MowerStatus {
  state: string;
  state_label: string;
  firmware: string;
  cleaning_count: number;
  total_area_m2: number;
  total_time_min: number;
  dnd: boolean;
  zone_id: string;
  zone_state: string;
  has_update: boolean;
}

export interface PhoneStatus {
  battery: number;
  charging: boolean;
  battery_state: string;
  charge_time_min: number;
  at_home: boolean;
  wifi_ssid: string;
  activity: string;
}

export interface CalendarEvent {
  summary: string;
  start: string;
  end: string;
  all_day: boolean;
  description: string;
  location: string;
}

export interface MediaInfo {
  title: string;
  artist: string;
  album: string;
  is_playing: boolean;
  thumbnail: string | null;
  duration_ms: number;
  view_offset_ms: number;
  source: "plex" | "gsmtc";
  session_id: string | null;
  machine_id: string | null;
  address: string | null;
}

export interface HourlyForecast {
  time: string;
  temp: number;
  symbol: string;
  precip: number;
}

export interface DailyForecast {
  date: string;
  temp_min: number;
  temp_max: number;
  symbol: string;
  humidity: number;
}

export interface WeatherDetailed {
  temp: number;
  symbol: string;
  precip: number;
  place_name: string;
  hourly: HourlyForecast[];
  daily: DailyForecast[];
}

export interface LocationResult {
  name: string;
  lat: number;
  lon: number;
  country: string;
  url_path: string;
}

export interface RateLimitWindow {
  label: string;
  usedPercent: number | null;
  remainingPercent: number | null;
  resetsAt: number | null;
}

export interface LimitProvider {
  enabled: boolean;
  ok: boolean;
  planType: string | null;
  shortWindow: RateLimitWindow;
  longWindow: RateLimitWindow;
  rateLimitReachedType: string | null;
  error: string | null;
}

export interface LimitsSnapshot {
  providers: Record<string, LimitProvider>;
}

export type ProjectCheckStatus = "pass" | "warn" | "fail" | "na";

export interface ProjectCheck {
  status: ProjectCheckStatus;
  detail: string;
}

export interface Project {
  name: string;
  fullName: string;
  description: string | null;
  url: string;
  isPrivate: boolean;
  isArchived: boolean;
  pushedAt: string | null;
  openIssuesCount: number;
  openPrsCount: number;
  releasesCount: number;
  healthScore: number;
  checks: Record<string, ProjectCheck>;
}

export interface ProjectsSnapshot {
  projects: Project[];
  averageHealth: number;
  attentionCount: number;
  fetchedAt: number;
}
