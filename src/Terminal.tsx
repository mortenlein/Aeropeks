import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "@xterm/xterm/css/xterm.css";
import { X } from "lucide-react";

function Terminal() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const unlistenRefs = useRef<(() => void)[]>([]);
  const ptyListenersRef = useRef<(() => void)[]>([]);

  const setupPty = async (args: string[] | null = null) => {
    const term = xtermRef.current;
    if (!term) return;

    try {
      // 1. Clear previous PTY-specific listeners
      ptyListenersRef.current.forEach(u => u());
      ptyListenersRef.current = [];

      // 2. Set up new PTY data listener
      const unPty = await listen<{ data: string }>("pty-data", (event) => {
        try {
          const binaryString = window.atob(event.payload.data);
          const bytes = new Uint8Array(binaryString.length);
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
          }
          term.write(bytes);
        } catch (e) {
          console.error("PTY Decode Error:", e);
        }
      });
      ptyListenersRef.current.push(unPty);

      // 3. Optional: other PTY events
      const unReady = await listen("pty-ready-global", () => { });
      ptyListenersRef.current.push(unReady);

      const unExit = await listen<string>("pty-exit", (event) => {
        term.write(`\r\n\x1b[33m[Process Terminated: ${event.payload}]\x1b[0m\r\n`);
      });
      ptyListenersRef.current.push(unExit);

      // 4. Start the backend PTY
      await invoke("start_pty", { rows: term.rows, cols: term.cols, args });
    } catch (e) {
      term.write(`\r\n\x1b[31mSetup Error: ${e}\x1b[0m\r\n`);
    }
  };

  useEffect(() => {
    if (!terminalRef.current) return;

    const accentColor = getComputedStyle(document.documentElement).getPropertyValue("--accent") || "#22c55e";

    const term = new XTerm({
      cursorBlink: true,
      theme: {
        background: "#08080c",
        foreground: "#e2e8f0",
        cursor: accentColor,
        selectionBackground: "rgba(34, 197, 94, 0.3)",
        black: "#000000",
        red: "#ef4444",
        green: "#22c55e",
        yellow: "#f59e0b",
        blue: "#3b82f6",
        magenta: "#d946ef",
        cyan: "#06b6d4",
        white: "#ffffff",
      },
      fontFamily: '"MesloLGS NF", "JetBrainsMono NF", Consolas, "Courier New", monospace',
      fontSize: 14,
      letterSpacing: 0,
      allowProposedApi: true,
      allowTransparency: true,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    document.fonts.ready.then(() => {
      if (!terminalRef.current) return;
      term.open(terminalRef.current);
      fitAddon.fit();
      setTimeout(() => term.refresh(0, term.rows - 1), 200);
    });

    xtermRef.current = term;

    // Initial shell
    setupPty();

    // Persistent Session Listener
    listen<{ data: string }>("start-session", (event) => {
      try {
        const args = JSON.parse(event.payload.data);
        term.reset();
        if (args.length > 0) {
          term.write(`\x1b[32mStarting session: ${args.join(" ")}\x1b[0m\r\n`);
        } else {
          term.write(`\x1b[32mStarting fresh local shell...\x1b[0m\r\n`);
        }
        setupPty(args);
      } catch (e) {
        console.error("Failed to parse session args:", e);
      }
    }).then(u => unlistenRefs.current.push(u));

    term.onData((data) => {
      invoke("write_pty", { data }).catch(() => { });
    });

    term.onResize(({ cols, rows }) => {
      invoke("resize_pty", { rows, cols }).catch(() => { });
    });

    const handleResize = () => {
      fitAddon.fit();
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
      unlistenRefs.current.forEach(u => u());
      ptyListenersRef.current.forEach(u => u());
    };
  }, []);

  const handleReset = () => {
    if (xtermRef.current) {
      xtermRef.current.reset();
      setupPty();
    }
  };

  return (
    <div className="terminal-outer">
      <div className="terminal-panel">
        <div className="terminal-header" data-tauri-drag-region>
          <div className="terminal-header-title">
            <span className="terminal-accent-dot" />
            Terminal
          </div>
          <div className="terminal-header-actions">
            <button className="header-action-btn" onClick={handleReset}>Reset</button>
            <button className="header-action-btn danger" onClick={() => invoke("kill_pty")}>Kill</button>
            <button className="close-btn" onClick={() => invoke("toggle_terminal_panel")}>
              <X size={14} />
            </button>
          </div>
        </div>
        <div ref={terminalRef} className="terminal-container" />
      </div>
    </div>
  );
}

export default Terminal;
