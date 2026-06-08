export type WindowType =
  | "main"
  | "settings"
  | "expanded-player"
  | "terminal-panel"
  | "launcher-panel"
  | "demo-weather"
  | "demo-usage"
  | "demo-projects";

export function normalizeWindowType(label: string): WindowType | null {
  if (label === "aeropeks") return "main";
  if (
    label === "main" ||
    label === "settings" ||
    label === "expanded-player" ||
    label === "terminal-panel" ||
    label === "launcher-panel" ||
    label === "demo-weather" ||
    label === "demo-usage" ||
    label === "demo-projects"
  ) {
    return label;
  }
  return null;
}
