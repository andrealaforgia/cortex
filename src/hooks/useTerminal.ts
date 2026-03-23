import { useEffect, useRef, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { useTerminalStore } from "../stores/terminalStore";
import {
  createTerminalSession,
  writeToPty,
  resizePty,
  closeTerminalSession,
  onPtyData,
  onPtyExit,
  onPtyError,
} from "../lib/tauri-bridge";

export function useTerminal(containerRef: React.RefObject<HTMLDivElement | null>) {
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const { sessionId, setSessionId, setError } = useTerminalStore();

  const initTerminal = useCallback(async () => {
    if (!containerRef.current || terminalRef.current) return;

    const terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Menlo', monospace",
      theme: {
        background: "#1e1e2e",
        foreground: "#cdd6f4",
        cursor: "#f5e0dc",
        selectionBackground: "#585b70",
        black: "#45475a",
        red: "#f38ba8",
        green: "#a6e3a1",
        yellow: "#f9e2af",
        blue: "#89b4fa",
        magenta: "#f5c2e7",
        cyan: "#94e2d5",
        white: "#bac2de",
        brightBlack: "#585b70",
        brightRed: "#f38ba8",
        brightGreen: "#a6e3a1",
        brightYellow: "#f9e2af",
        brightBlue: "#89b4fa",
        brightMagenta: "#f5c2e7",
        brightCyan: "#94e2d5",
        brightWhite: "#a6adc8",
      },
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(new WebLinksAddon());

    terminal.open(containerRef.current);

    try {
      const webglAddon = new WebglAddon();
      terminal.loadAddon(webglAddon);
    } catch {
      console.warn("WebGL addon failed to load, falling back to canvas");
    }

    fitAddon.fit();

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // Create PTY session
    try {
      const { session_id } = await createTerminalSession();
      setSessionId(session_id);

      // Forward terminal input to PTY
      terminal.onData((data) => {
        const bytes = Array.from(new TextEncoder().encode(data));
        writeToPty(session_id, bytes);
      });

      // Handle resize
      terminal.onResize(({ cols, rows }) => {
        resizePty(session_id, rows, cols);
      });

      // Listen for PTY output
      const unlistenData = await onPtyData((payload) => {
        if (payload.session_id === session_id) {
          const text = new TextDecoder().decode(new Uint8Array(payload.data));
          terminal.write(text);
        }
      });

      // Listen for PTY exit
      const unlistenExit = await onPtyExit((payload) => {
        if (payload.session_id === session_id) {
          terminal.write(`\r\n[Process exited with code ${payload.code}]\r\n`);
        }
      });

      // Listen for PTY errors
      const unlistenError = await onPtyError((payload) => {
        if (payload.session_id === session_id) {
          setError(`Terminal error: ${payload.message}`);
          terminal.write(
            `\r\n\x1b[31m[Error: ${payload.message}]\x1b[0m\r\n`
          );
        }
      });

      // Cleanup function stored for later
      (terminal as any)._cleanup = () => {
        unlistenData();
        unlistenExit();
        unlistenError();
        closeTerminalSession(session_id);
      };
    } catch (err) {
      setError(`Failed to create terminal session: ${err}`);
    }

    // Handle window resize
    const handleResize = () => fitAddon.fit();
    window.addEventListener("resize", handleResize);
    (terminal as any)._resizeCleanup = () =>
      window.removeEventListener("resize", handleResize);

    return terminal;
  }, [containerRef, setSessionId, setError]);

  useEffect(() => {
    initTerminal();

    return () => {
      const terminal = terminalRef.current;
      if (terminal) {
        (terminal as any)._cleanup?.();
        (terminal as any)._resizeCleanup?.();
        terminal.dispose();
        terminalRef.current = null;
      }
    };
  }, [initTerminal]);

  return { terminal: terminalRef, fitAddon: fitAddonRef, sessionId };
}
