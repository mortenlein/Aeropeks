import { lazy, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { normalizeWindowType } from "./window-routing";
import "./index.css";

const App = lazy(() => import("./App"));
const Settings = lazy(() => import("./Settings"));
const ExpandedPlayer = lazy(() => import("./ExpandedPlayer"));
const Terminal = lazy(() => import("./Terminal"));
const Launcher = lazy(() => import("./Launcher"));
const DemoWeather = lazy(() =>
  import("./DemoPopup").then((m) => ({ default: m.DemoWeather })),
);
const DemoUsage = lazy(() =>
  import("./DemoPopup").then((m) => ({ default: m.DemoUsage })),
);
const DemoProjects = lazy(() =>
  import("./DemoPopup").then((m) => ({ default: m.DemoProjects })),
);

function resolveWindowType() {
  const requested = new URLSearchParams(window.location.search).get("window");
  return normalizeWindowType(requested || getCurrentWindow().label);
}

function Root() {
  switch (resolveWindowType()) {
    case "main":
      return <App />;
    case "settings":
      return <Settings />;
    case "expanded-player":
      return <ExpandedPlayer />;
    case "terminal-panel":
      return <Terminal />;
    case "launcher-panel":
      return <Launcher />;
    case "demo-weather":
      return <DemoWeather />;
    case "demo-usage":
      return <DemoUsage />;
    case "demo-projects":
      return <DemoProjects />;
    default:
      return null;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <Suspense fallback={null}>
    <Root />
  </Suspense>,
);
