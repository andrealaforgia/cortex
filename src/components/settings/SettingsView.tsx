import { useState, useEffect } from "react";
import { storeApiKey, hasApiKey, getConfig, setConfig } from "../../lib/tauri-bridge";

interface SettingsViewProps {
  onClose: () => void;
}

const STYLES: Record<string, React.CSSProperties> = {
  overlay: {
    position: "fixed",
    inset: 0,
    background: "rgba(0,0,0,0.6)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    zIndex: 100,
  },
  panel: {
    background: "#1e1e2e",
    border: "1px solid #313244",
    borderRadius: "12px",
    width: "480px",
    maxHeight: "80vh",
    overflow: "auto",
    padding: "24px",
    color: "#cdd6f4",
    fontFamily: "'Inter', -apple-system, sans-serif",
    fontSize: "13px",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: "24px",
    fontSize: "18px",
    fontWeight: 600,
  },
  section: {
    marginBottom: "20px",
  },
  label: {
    display: "block",
    marginBottom: "6px",
    color: "#a6adc8",
    fontSize: "12px",
    fontWeight: 500,
  },
  input: {
    width: "100%",
    background: "#313244",
    border: "1px solid #45475a",
    borderRadius: "6px",
    padding: "8px 12px",
    color: "#cdd6f4",
    fontSize: "13px",
    outline: "none",
    fontFamily: "monospace",
  },
  button: {
    background: "#89b4fa",
    border: "none",
    borderRadius: "6px",
    padding: "8px 16px",
    color: "#1e1e2e",
    cursor: "pointer",
    fontWeight: 600,
    fontSize: "13px",
    marginTop: "8px",
  },
  status: {
    marginTop: "8px",
    fontSize: "12px",
    padding: "6px 10px",
    borderRadius: "4px",
  },
};

export default function SettingsView({ onClose }: SettingsViewProps) {
  const [apiKey, setApiKeyInput] = useState("");
  const [hasKey, setHasKey] = useState(false);
  const [status, setStatus] = useState<{ type: "success" | "error"; msg: string } | null>(null);
  const [saving, setSaving] = useState(false);
  const [defaultModel, setDefaultModel] = useState("claude-sonnet-4-20250514");

  useEffect(() => {
    hasApiKey().then((r) => setHasKey(r.exists));
    getConfig().then((cfg) => {
      if (cfg.default_model) setDefaultModel(cfg.default_model);
    });
  }, []);

  const handleSaveKey = async () => {
    if (!apiKey.trim()) return;
    setSaving(true);
    setStatus(null);
    try {
      const result = await storeApiKey(apiKey.trim());
      if (result.valid) {
        setHasKey(true);
        setApiKeyInput("");
        setStatus({ type: "success", msg: "API key validated and saved" });
      } else {
        setStatus({ type: "error", msg: result.error || "Invalid API key" });
      }
    } catch (err) {
      setStatus({ type: "error", msg: `Error: ${err}` });
    }
    setSaving(false);
  };

  const handleSaveModel = async () => {
    try {
      await setConfig({ default_model: defaultModel });
      setStatus({ type: "success", msg: "Settings saved" });
    } catch (err) {
      setStatus({ type: "error", msg: `Error: ${err}` });
    }
  };

  return (
    <div style={STYLES.overlay} onClick={onClose}>
      <div style={STYLES.panel} onClick={(e) => e.stopPropagation()}>
        <div style={STYLES.header}>
          <span>Settings</span>
          <button
            onClick={onClose}
            style={{ background: "none", border: "none", color: "#6c7086", cursor: "pointer", fontSize: "20px" }}
          >
            ×
          </button>
        </div>

        <div style={STYLES.section}>
          <label style={STYLES.label}>
            Anthropic API Key {hasKey && <span style={{ color: "#a6e3a1" }}>✓ Configured</span>}
          </label>
          <input
            style={STYLES.input}
            type="password"
            placeholder={hasKey ? "••••••••••••• (saved)" : "sk-ant-..."}
            value={apiKey}
            onChange={(e) => setApiKeyInput(e.target.value)}
          />
          <button style={STYLES.button} onClick={handleSaveKey} disabled={saving}>
            {saving ? "Validating..." : "Save Key"}
          </button>
        </div>

        <div style={STYLES.section}>
          <label style={STYLES.label}>Default Model</label>
          <select
            style={{ ...STYLES.input, cursor: "pointer" }}
            value={defaultModel}
            onChange={(e) => setDefaultModel(e.target.value)}
          >
            <option value="claude-sonnet-4-20250514">Claude Sonnet 4 (balanced)</option>
            <option value="claude-haiku-4-5-20251001">Claude Haiku 4.5 (fast)</option>
            <option value="claude-opus-4-20250514">Claude Opus 4 (powerful)</option>
          </select>
          <button style={STYLES.button} onClick={handleSaveModel}>
            Save
          </button>
        </div>

        {status && (
          <div
            style={{
              ...STYLES.status,
              background: status.type === "success" ? "#1e3a2f" : "#3a1e2f",
              color: status.type === "success" ? "#a6e3a1" : "#f38ba8",
            }}
          >
            {status.msg}
          </div>
        )}

        <div style={{ marginTop: "24px", color: "#585b70", fontSize: "11px" }}>
          <p>Keyboard shortcuts:</p>
          <p>Ctrl+Shift+A — Toggle AI panel</p>
          <p>Ctrl+, — Open settings</p>
          <p style={{ marginTop: "8px" }}>
            API key is stored in memory. Set ANTHROPIC_API_KEY env var for persistence.
          </p>
        </div>
      </div>
    </div>
  );
}
