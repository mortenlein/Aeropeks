import { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import Settings from "./Settings";
import ExpandedPlayer from "./ExpandedPlayer";
import Terminal from "./Terminal";
import Launcher from "./Launcher";
import "./index.css";

function Root() {
  const [windowType, setWindowType] = useState<string | null>(null);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const type = params.get("window") || getCurrentWindow().label;
    console.log("Window Type identified as:", type);
    setWindowType(type);
  }, []);

  // Show nothing while identifying to prevent ghost layouts
  if (!windowType) return null;

  if (windowType === "settings") {
    return <Settings />;
  }

  if (windowType === "expanded-player") {
    return <ExpandedPlayer />;
  }

  if (windowType === "terminal-panel") {
    return <Terminal />;
  }
  
  if (windowType === "launcher-panel") {
    return <Launcher />;
  }

  // Only render App for the 'main' window explicitly
  if (windowType === "main" || windowType === "aeropeks") {
    return <App />;
  }

  return null;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<Root />);
