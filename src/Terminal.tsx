import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "@xterm/xterm/css/xterm.css";
import { Icon } from "./icons";

function Terminal() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const unlistenRefs = useRef<(() => void)[]>([]);
  const ptyListenersRef = useRef<(() => void)[]>([]);

  const setupPty = async (command: string | null = null) => {
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
      await invoke("start_pty", { rows: term.rows, cols: term.cols, command });
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
        background:   "transparent",
        foreground:   "#E4E8EC",
        cursor:       accentColor,
        cursorAccent: "#0e1013",
        selectionBackground: "rgba(34, 197, 94, 0.25)",
        black:        "#0e1013",
        red:          "#D96A5F",
        green:        "#22C55E",
        yellow:       "#D9A93F",
        blue:         "#5E8FD8",
        magenta:      "#A887E0",
        cyan:         "#54AEC8",
        white:        "#A0AAB6",
        brightBlack:  "#767F90",
        brightRed:    "#E07A70",
        brightGreen:  "#3DD96E",
        brightYellow: "#E6B84C",
        brightBlue:   "#709EE5",
        brightMagenta:"#B899E8",
        brightCyan:   "#62BCCE",
        brightWhite:  "#B4BEC8",
      },
      fontFamily: '"JetBrains Mono", "MesloLGS NF", ui-monospace, monospace',
      fontSize: 11.5,
      lineHeight: 1.6,
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
        const command = JSON.parse(event.payload.data) as string;
        term.reset();
        if (command) {
          term.write(`\x1b[32mStarting session: ${command}\x1b[0m\r\n`);
        } else {
          term.write(`\x1b[32mStarting fresh local shell...\x1b[0m\r\n`);
        }
        setupPty(command);
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
              <Icon name="close" size={12} />
            </button>
          </div>
        </div>
        <div ref={terminalRef} className="terminal-container" />
      </div>
    </div>
  );
}

export default Terminal;
