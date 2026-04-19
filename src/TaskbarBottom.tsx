import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AppWindow } from "lucide-react";

interface WindowInfo {
  hwnd: number;
  title: string;
  app_name: string;
  is_active: boolean;
  icon: string | null;
  icon_source?: string;
  identity_key?: string;
}

const TaskbarBottom = () => {
  const [windows, setWindows] = useState<WindowInfo[]>([]);

  const fetchWindows = async () => {
    try {
      const winList = await invoke<WindowInfo[]>("get_open_windows");
      // Filter out duplicate processes and prefer windows with titles
      const uniqueWindows = winList.filter((win, index, self) =>
        index === self.findIndex((t) => t.hwnd === win.hwnd)
      );
      setWindows(uniqueWindows);
    } catch (e) {
      console.error("Failed to fetch windows", e);
    }
  };

  useEffect(() => {
    fetchWindows();
    let unlisten: (() => void) | undefined;

    listen<WindowInfo[]>("open-windows-changed", (event) => {
      const uniqueWindows = event.payload.filter((win, index, self) =>
        index === self.findIndex((t) => t.hwnd === win.hwnd)
      );
      setWindows(uniqueWindows);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const handleFocus = async (hwnd: number) => {
    try {
      await invoke("focus_window", { hwnd });
      fetchWindows();
    } catch (e) {
      console.error("Failed to focus window", e);
    }
  };

  return (
    <div className="taskbar-bottom-container">
      <div className="taskbar-dock">
        <div className="dock-section apps">
          {windows.map((win) => (
            <div
              key={win.hwnd}
              className={`dock-item ${win.is_active ? "active" : ""}`}
              onClick={() => handleFocus(win.hwnd)}
              title={`${win.title}\n${win.icon_source || "icon source unknown"}`}
            >
              <div className="dock-icon">
                {win.icon ? (
                  <img src={win.icon} alt={win.app_name} width={24} height={24} />
                ) : (
                  <AppWindow size={20} />
                )}
              </div>
              <div className="dock-indicator" />
              <div className="dock-tooltip">{win.app_name || "App"}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default TaskbarBottom;
