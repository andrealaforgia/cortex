export interface Block {
  id: string;
  sessionId: string;
  command: string;
  output: string;
  exitCode: number | null;
  cwd: string;
  startedAt: string;
  finishedAt: string | null;
  durationMs: number | null;
  state: "prompting" | "running" | "completed";
}

export interface ShellContext {
  shell_type: "bash" | "zsh" | "fish" | "unknown";
  os: string;
  cwd: string;
  recent_commands: CommandEntry[];
  env_snippet?: string;
}

export interface CommandEntry {
  command: string;
  exit_code: number;
  output_preview?: string;
}

export interface CommandSuggestion {
  command: string;
  explanation: string;
  risk_level: "safe" | "moderate" | "dangerous";
}

export interface AppConfig {
  default_shell: string;
  default_model: string;
  scrollback_lines: number;
  theme: "dark" | "light";
  font_size: number;
  font_family: string;
  ai_context_lines: number;
  redaction_enabled: boolean;
  custom_redaction_patterns: string[];
}

export interface AIMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  model?: string;
  createdAt: string;
}

export interface AIConversation {
  id: string;
  title: string;
  messages: AIMessage[];
  createdAt: string;
  updatedAt: string;
}
