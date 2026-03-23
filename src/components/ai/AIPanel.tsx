import { useState, useRef, useEffect } from "react";
import { useAIStore } from "../../stores/aiStore";
import { aiChat, aiTranslateCommand, onAiStreamChunk, onAiStreamEnd, onAiError } from "../../lib/tauri-bridge";
import type { ShellContext } from "../../lib/types";

const PANEL_STYLES: Record<string, React.CSSProperties> = {
  container: {
    width: "380px",
    height: "100%",
    background: "#181825",
    borderLeft: "1px solid #313244",
    display: "flex",
    flexDirection: "column",
    fontFamily: "'Inter', -apple-system, sans-serif",
    fontSize: "13px",
    color: "#cdd6f4",
  },
  header: {
    padding: "12px 16px",
    borderBottom: "1px solid #313244",
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    fontWeight: 600,
  },
  messages: {
    flex: 1,
    overflowY: "auto" as const,
    padding: "12px 16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "12px",
  },
  userMsg: {
    background: "#313244",
    borderRadius: "8px",
    padding: "8px 12px",
    alignSelf: "flex-end" as const,
    maxWidth: "85%",
    whiteSpace: "pre-wrap" as const,
  },
  aiMsg: {
    background: "#1e1e2e",
    borderRadius: "8px",
    padding: "8px 12px",
    alignSelf: "flex-start" as const,
    maxWidth: "85%",
    whiteSpace: "pre-wrap" as const,
    borderLeft: "3px solid #89b4fa",
  },
  inputArea: {
    padding: "12px 16px",
    borderTop: "1px solid #313244",
    display: "flex",
    gap: "8px",
  },
  input: {
    flex: 1,
    background: "#313244",
    border: "1px solid #45475a",
    borderRadius: "6px",
    padding: "8px 12px",
    color: "#cdd6f4",
    fontSize: "13px",
    outline: "none",
    resize: "none" as const,
    fontFamily: "inherit",
  },
  sendBtn: {
    background: "#89b4fa",
    border: "none",
    borderRadius: "6px",
    padding: "8px 16px",
    color: "#1e1e2e",
    cursor: "pointer",
    fontWeight: 600,
    fontSize: "13px",
  },
};

interface Message {
  role: "user" | "assistant";
  content: string;
}

interface AIPanelProps {
  onClose: () => void;
}

export default function AIPanel({ onClose }: AIPanelProps) {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const { isStreaming, streamingText, setStreaming, appendStreamText, clearStreamText } = useAIStore();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unsubChunk = onAiStreamChunk((payload) => {
      appendStreamText(payload.text);
    });
    const unsubEnd = onAiStreamEnd(() => {
      setStreaming(false);
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: useAIStore.getState().streamingText },
      ]);
      clearStreamText();
    });
    const unsubError = onAiError((payload) => {
      setStreaming(false);
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: `Error: ${payload.message}` },
      ]);
      clearStreamText();
    });

    return () => {
      unsubChunk.then((fn) => fn());
      unsubEnd.then((fn) => fn());
      unsubError.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText]);

  const handleSend = async () => {
    const trimmed = input.trim();
    if (!trimmed || isStreaming) return;

    setMessages((prev) => [...prev, { role: "user", content: trimmed }]);
    setInput("");
    setStreaming(true);
    clearStreamText();

    const context: ShellContext = {
      shell_type: "zsh",
      os: "macOS",
      cwd: "~",
      recent_commands: [],
    };

    try {
      if (trimmed.startsWith("/cmd ")) {
        await aiTranslateCommand(trimmed.slice(5), context);
      } else {
        await aiChat(trimmed, null, context);
      }
    } catch (err) {
      setStreaming(false);
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: `Failed to send: ${err}` },
      ]);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div style={PANEL_STYLES.container}>
      <div style={PANEL_STYLES.header}>
        <span>AI Assistant</span>
        <button
          onClick={onClose}
          style={{
            background: "none",
            border: "none",
            color: "#6c7086",
            cursor: "pointer",
            fontSize: "18px",
          }}
        >
          ×
        </button>
      </div>

      <div style={PANEL_STYLES.messages}>
        {messages.length === 0 && !isStreaming && (
          <div style={{ color: "#6c7086", textAlign: "center", padding: "24px" }}>
            Ask anything about your terminal.
            <br />
            <span style={{ fontSize: "12px" }}>
              Use <code>/cmd</code> to translate natural language to commands.
            </span>
          </div>
        )}

        {messages.map((msg, i) => (
          <div
            key={i}
            style={msg.role === "user" ? PANEL_STYLES.userMsg : PANEL_STYLES.aiMsg}
          >
            {msg.content}
          </div>
        ))}

        {isStreaming && streamingText && (
          <div style={PANEL_STYLES.aiMsg}>
            {streamingText}
            <span style={{ opacity: 0.5 }}>▊</span>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <div style={PANEL_STYLES.inputArea}>
        <textarea
          style={PANEL_STYLES.input}
          rows={1}
          placeholder="Ask AI... (/cmd for commands)"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={isStreaming}
        />
        <button
          style={{
            ...PANEL_STYLES.sendBtn,
            opacity: isStreaming ? 0.5 : 1,
          }}
          onClick={handleSend}
          disabled={isStreaming}
        >
          Send
        </button>
      </div>
    </div>
  );
}
