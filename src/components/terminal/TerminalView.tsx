import { useRef } from "react";
import { useTerminal } from "../../hooks/useTerminal";
import { useTerminalStore } from "../../stores/terminalStore";

export default function TerminalView() {
  const containerRef = useRef<HTMLDivElement>(null);
  useTerminal(containerRef);
  const error = useTerminalStore((s) => s.error);

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      {error && (
        <div
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            padding: "8px 16px",
            background: "#f38ba8",
            color: "#1e1e2e",
            fontSize: "13px",
            zIndex: 10,
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <span>{error}</span>
          <button
            onClick={() => useTerminalStore.getState().setError(null)}
            style={{
              background: "none",
              border: "none",
              color: "#1e1e2e",
              cursor: "pointer",
              fontSize: "16px",
            }}
          >
            ×
          </button>
        </div>
      )}
      <div
        ref={containerRef}
        style={{
          width: "100%",
          height: "100%",
          padding: "4px",
        }}
      />
    </div>
  );
}
