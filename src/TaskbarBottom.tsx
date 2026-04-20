import { useEffect, useMemo, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ShoppingBag, Folder, FileText, Settings, AppWindow, Search, LayoutGrid } from "lucide-react";

interface WindowInfo {
  hwnd: number;
  title: string;
  app_name: string;
  is_active: boolean;
  icon: string | null;
  icon_source?: string;
  identity_key?: string;
}

interface DockGroup {
  key: string;
  appName: string;
  icon: string | null;
  iconSource?: string;
  windows: WindowInfo[];
  isActive: boolean;
}

const getGroupKey = (win: WindowInfo) =>
  win.identity_key || win.app_name || win.title || `hwnd:${win.hwnd}`;

const groupWindows = (windows: WindowInfo[]): DockGroup[] => {
  const groups = new Map<string, DockGroup>();

  windows.forEach((win) => {
    const key = getGroupKey(win);
    const existing = groups.get(key);

    if (existing) {
      existing.windows.push(win);
      existing.isActive = existing.isActive || win.is_active;
      if (win.is_active || !existing.icon) {
        existing.icon = win.icon;
        existing.iconSource = win.icon_source;
      }
      return;
    }

    groups.set(key, {
      key,
      appName: win.app_name || "App",
      icon: win.icon,
      iconSource: win.icon_source,
      windows: [win],
      isActive: win.is_active,
    });
  });

  return Array.from(groups.values());
};

const TaskbarBottom = () => {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [desktopCount, setDesktopCount] = useState(1);
  const [currentDesktop, setCurrentDesktop] = useState(0);
  const [thumbnails, setThumbnails] = useState<Record<number, string | null>>({});
  const [hoveredGroup, setHoveredGroup] = useState<string | null>(null);
  const timeoutRef = useMemo(() => ({ entry: null as any, exit: null as any, refresh: null as any }), []);

  const groupedWindows = useMemo(() => groupWindows(windows), [windows]);

  const fetchWindows = async () => {
    try {
      const winList = await invoke<WindowInfo[]>("get_open_windows");
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

  const handleGroupClick = async (group: DockGroup) => {
    const target = group.windows.find((win) => win.is_active) || group.windows[0];
    if (target) {
      await handleFocus(target.hwnd);
    }
  };

  const handleCloseWindow = async (e: React.MouseEvent, hwnd: number) => {
    e.stopPropagation();
    try {
      await invoke("close_window", { hwnd });
      fetchWindows();
    } catch (e) {
      console.error("Failed to close window", e);
    }
  };

  const loadGroupThumbnails = (group: DockGroup, force = false) => {
    group.windows.forEach((win) => {
      if (!force && thumbnails.hasOwnProperty(win.hwnd)) return;

      invoke<string | null>("get_window_thumbnail", { hwnd: win.hwnd })
        .then((thumbnail) => {
          setThumbnails((current) => ({ ...current, [win.hwnd]: thumbnail }));
        })
        .catch(() => {});
    });
  };
  // Ensure body/html don't clip the preview popup above the dock
  useEffect(() => {
    document.body.style.overflow = 'visible';
    document.documentElement.style.overflow = 'visible';
    document.documentElement.style.height = '100%';
    document.body.style.height = '100%';
    const root = document.getElementById('root');
    if (root) {
      root.style.overflow = 'visible';
      root.style.height = '100%';
    }
  }, []);

  const resizeWindow = useCallback(async (expanded: boolean) => {
    try {
      const win = getCurrentWindow();
      const monitor = await win.currentMonitor();
      if (!monitor) return;
      const screenH = monitor.size.height;
      const screenW = monitor.size.width;
      const h = expanded ? 300 : 60;
      const { PhysicalSize, PhysicalPosition } = await import("@tauri-apps/api/dpi");
      await win.setSize(new PhysicalSize(screenW, h));
      await win.setPosition(new PhysicalPosition(0, screenH - h));
    } catch (e) {
      console.error("Failed to resize taskbar window", e);
    }
  }, []);

  const handleMouseEnter = (group: DockGroup) => {
    if (timeoutRef.exit) clearTimeout(timeoutRef.exit);
    if (timeoutRef.refresh) clearInterval(timeoutRef.refresh);
    
    timeoutRef.entry = setTimeout(async () => {
      // 1. Tell Rust thread to stop overriding the window position
      invoke("set_preview_mode", { active: true }).catch(() => {});
      // 2. Expand window FIRST so there's room for the preview
      await resizeWindow(true);
      // 3. Now show the preview (React re-render)
      setHoveredGroup(group.key);
      loadGroupThumbnails(group);
      
      timeoutRef.refresh = setInterval(() => {
        loadGroupThumbnails(group, true);
      }, 3000);
    }, 250); 
  };

  const handleMouseLeave = () => {
    if (timeoutRef.entry) clearTimeout(timeoutRef.entry);
    if (timeoutRef.refresh) clearInterval(timeoutRef.refresh);
    
    timeoutRef.exit = setTimeout(async () => {
      setHoveredGroup(null);
      // Shrink window back and re-enable thread positioning
      await resizeWindow(false);
      invoke("set_preview_mode", { active: false }).catch(() => {});
    }, 400);
  };

  return (
    <div className="taskbar-bottom-container">
      <div className="taskbar-dock">
        {/* Left Section: Pinned/System */}
        <div className="dock-section fixed">
          <div className="dock-item system" title="Launcher">
            <LayoutGrid size={20} />
            <div className="dock-tooltip">Launcher</div>
          </div>
          <div className="dock-item system" title="Search">
            <Search size={20} />
            <div className="dock-tooltip">Search</div>
          </div>
          <div className="dock-divider" />
          <div className="dock-item" title="Microsoft Store">
            <ShoppingBag size={20} />
            <div className="dock-indicator dot" />
            <div className="dock-tooltip">Microsoft Store</div>
          </div>
          <div className="dock-item" title="File Explorer">
            <Folder size={20} />
            <div className="dock-indicator dot" />
            <div className="dock-tooltip">File Explorer</div>
          </div>
          <div className="dock-item" title="Notepad">
            <FileText size={20} />
            <div className="dock-tooltip">Notepad</div>
          </div>
        </div>

        <div className="dock-divider" />

        {/* Center Section: Open Apps */}
        <div className="dock-section apps">
          {groupedWindows.map((group) => (
            <div
              key={group.key}
              className={`dock-item ${group.isActive ? "active" : ""} ${group.windows.length > 1 ? "grouped" : ""} ${hoveredGroup === group.key ? "hover" : ""}`}
              onClick={() => handleGroupClick(group)}
              onMouseEnter={() => handleMouseEnter(group)}
              onMouseLeave={handleMouseLeave}
            >
              <div className="dock-icon">
                {group.icon ? (
                  <img src={group.icon} alt={group.appName} width={24} height={24} />
                ) : (
                  <AppWindow size={20} />
                )}
              </div>
              {group.windows.length > 1 && (
                <div className="dock-badge">{group.windows.length}</div>
              )}
              <div className={`dock-indicator ${group.isActive ? "dash" : "dot"}`} />
              <div className="dock-tooltip">{group.appName}</div>
              
              <div className={`dock-preview-strip ${hoveredGroup === group.key ? "visible" : ""}`} onClick={(e) => e.stopPropagation()}>
                {group.windows.map((win) => (
                  <div
                    key={win.hwnd}
                    className={`dock-preview-card ${win.is_active ? "active" : ""}`}
                    onClick={() => handleFocus(win.hwnd)}
                  >
                    <div className="preview-header">
                      <div className="preview-app-info">
                        {win.icon && <img src={win.icon} alt="" className="mini-icon" />}
                        <span className="preview-app-name">{win.app_name || group.appName}</span>
                      </div>
                      <button className="preview-close" onClick={(e) => handleCloseWindow(e, win.hwnd)}>
                        <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                      </button>
                    </div>
                    <div className="preview-thumb">
                      {thumbnails[win.hwnd] ? (
                        <img src={thumbnails[win.hwnd] || ""} alt={win.title} />
                      ) : (
                        <div className="preview-fallback">
                          {win.icon ? (
                            <img src={win.icon} alt={win.app_name} />
                          ) : (
                            <AppWindow size={24} />
                          )}
                        </div>
                      )}
                    </div>
                    <div className="preview-copy">
                      <span className="preview-title">{win.title}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div className="dock-divider" />

        {/* Right Section: Tray/Desktops */}
        <div className="dock-section tray">
          {desktopCount > 1 && (
            <div className="desktops-pill">
              {Array.from({ length: desktopCount }).map((_, idx) => (
                <div 
                  key={idx}
                  title={`Desktop ${idx + 1}`}
                  className={`desktop-dot ${idx === currentDesktop ? 'active' : ''}`}
                  onClick={() => invoke("switch_virtual_desktop", { index: idx }).then(() => setCurrentDesktop(idx))}
                />
              ))}
            </div>
          )}
          <div className="dock-item system" title="Settings" onClick={() => invoke("open_settings")}>
            <Settings size={20} />
            <div className="dock-indicator dot active" />
            <div className="dock-tooltip">Settings</div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TaskbarBottom;
