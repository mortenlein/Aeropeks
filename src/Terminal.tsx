import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "@xterm/xterm/css/xterm.css";
import { X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

function Terminal() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const unlistenRefs = useRef<(() => void)[]>([]);

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
      fontFamily: 'Consolas, "Courier New", monospace',
      fontSize: 14,
      allowTransparency: true,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalRef.current);
    fitAddon.fit();

    xtermRef.current = term;

    const handlePtyData = (b64: string) => {
      try {
        const binaryString = window.atob(b64);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
          bytes[i] = binaryString.charCodeAt(i);
        }
        term.write(bytes);
      } catch (e) {
        console.error("PTY Decode Error:", e);
      }
    };


    const setupPty = async () => {
      try {
        term.write("Registering event listeners...\r\n");
        
        const unPty = await listen<{data: string}>("pty-data", (event) => {
          term.write("\x1b[34m[D]\x1b[0m");
          handlePtyData(event.payload.data);
        });
        unlistenRefs.current.push(unPty);

        const unReady = await listen("pty-ready-global", () => {
          term.write("\x1b[32m[READY]\x1b[0m\r\n");
        });
        unlistenRefs.current.push(unReady);

        const unHB = await listen("pty-heartbeat", () => {
          term.write("\x1b[35m[HB]\x1b[0m");
        });
        unlistenRefs.current.push(unHB);

        term.write("Invoking start_pty (Targeted)...\r\n");
        await invoke("start_pty", { rows: term.rows, cols: term.cols });
        term.write("Backend initialized successfully.\r\n");
      } catch (e) {
        term.write(`\r\n\x1b[31mSetup Error: ${e}\x1b[0m\r\n`);
        console.error("PTY Setup Error:", e);
      }
    };

    setupPty();

    term.onData((data) => {
      invoke("write_pty", { data }).catch(() => {});
    });

    term.onResize(({ cols, rows }) => {
      invoke("resize_pty", { rows, cols }).catch(() => {});
    });

    const handleResize = () => {
      fitAddon.fit();
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
      unlistenRefs.current.forEach(u => u());
    };
  }, []);

  return (
    <div className="terminal-outer">
      <div className="terminal-panel">
        <div className="terminal-header" data-tauri-drag-region>
          <div className="terminal-header-title">
            <span className="terminal-accent-dot" />
            Terminal
          </div>
          <button className="close-btn" onClick={() => getCurrentWindow().hide()}>
            <X size={14} />
          </button>
        </div>
        <div ref={terminalRef} className="terminal-container" />
      </div>
    </div>
  );
}

export default Terminal;
