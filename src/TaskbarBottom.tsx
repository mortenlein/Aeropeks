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
  const [desktopCount, setDesktopCount] = useState(1);
  const [currentDesktop, setCurrentDesktop] = useState(0);

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
    
    const fetchDesktops = async () => {
      try {
        const [count, index] = await invoke<[number, number]>("get_virtual_desktop_status");
        setDesktopCount(count);
        setCurrentDesktop(index);
      } catch(e) {}
    };
    
    fetchDesktops();
    const desktopInterval = setInterval(fetchDesktops, 1500);

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
      clearInterval(desktopInterval);
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
        {desktopCount > 1 && (
          <>
            <div className="dock-section desktops">
              {Array.from({ length: desktopCount }).map((_, idx) => (
                <div 
                  key={idx}
                  title={`Desktop ${idx + 1}`}
                  className={`desktop-dot ${idx === currentDesktop ? 'active' : ''}`}
                  onClick={() => invoke("switch_virtual_desktop", { index: idx }).then(() => setCurrentDesktop(idx))}
                />
              ))}
            </div>
            <div className="dock-divider" />
          </>
        )}
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
