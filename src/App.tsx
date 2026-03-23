import { useState, useEffect } from "react";
import TerminalView from "./components/terminal/TerminalView";
import AIPanel from "./components/ai/AIPanel";
import SettingsView from "./components/settings/SettingsView";

export default function App() {
  const [showAI, setShowAI] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === "A") {
        e.preventDefault();
        setShowAI((prev) => !prev);
      }
      if (e.ctrlKey && e.key === ",") {
        e.preventDefault();
        setShowSettings((prev) => !prev);
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
      {showSettings && <SettingsView onClose={() => setShowSettings(false)} />}
    </div>
  );
}
