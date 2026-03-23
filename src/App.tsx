import { useState, useEffect } from "react";
import TerminalView from "./components/terminal/TerminalView";
import AIPanel from "./components/ai/AIPanel";

export default function App() {
  const [showAI, setShowAI] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Shift+A toggles AI panel
      if (e.ctrlKey && e.shiftKey && e.key === "A") {
        e.preventDefault();
        setShowAI((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        background: "#1e1e2e",
        display: "flex",
      }}
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <TerminalView />
      </div>
      {showAI && <AIPanel onClose={() => setShowAI(false)} />}
    </div>
  );
}
