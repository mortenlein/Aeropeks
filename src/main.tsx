import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import Settings from "./Settings";
import "./index.css";

function Root() {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    setLabel(getCurrentWindow().label);
  }, []);

  if (!label) return null;

  return (
    <React.StrictMode>
      {label === "main" ? <App /> : <Settings />}
    </React.StrictMode>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<Root />);
