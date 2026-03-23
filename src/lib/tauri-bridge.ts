import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { ShellContext } from "./types";

// Terminal commands
export async function createTerminalSession(
  shell?: string,
  cwd?: string
): Promise<{ session_id: string }> {
  return invoke("create_terminal_session", { shell, cwd });
}

export async function writeToPty(
  sessionId: string,
  data: number[]
): Promise<void> {
  return invoke("write_to_pty", { sessionId, data });
}

export async function resizePty(
  sessionId: string,
  rows: number,
  cols: number
): Promise<void> {
  return invoke("resize_pty", { sessionId, rows, cols });
}

export async function closeTerminalSession(
  sessionId: string
): Promise<void> {
  return invoke("close_terminal_session", { sessionId });
}

// AI commands
export async function aiTranslateCommand(
  query: string,
  context: ShellContext
): Promise<{ request_id: string }> {
  return invoke("ai_translate_command", { query, context });
}

export async function aiExplainError(
  command: string,
  output: string,
  exitCode: number,
  context: ShellContext
): Promise<{ request_id: string }> {
  return invoke("ai_explain_error", { command, output, exitCode, context });
}

export async function aiChat(
  message: string,
  conversationId: string | null,
  context: ShellContext
): Promise<{ request_id: string; conversation_id: string }> {
  return invoke("ai_chat", { message, conversationId, context });
}

export async function aiCancel(requestId: string): Promise<void> {
  return invoke("ai_cancel", { requestId });
}

// Config commands
export async function getConfig(): Promise<Record<string, string>> {
  return invoke("get_config");
}

export async function setConfig(
  config: Record<string, string>
): Promise<void> {
  return invoke("set_config", { config });
}

export async function storeApiKey(
  key: string
): Promise<{ valid: boolean; error?: string }> {
  return invoke("store_api_key", { key });
}

export async function hasApiKey(): Promise<{ exists: boolean }> {
  return invoke("has_api_key");
}

// Event listeners
export interface PtyDataPayload {
  session_id: string;
  data: number[];
}

export interface PtyExitPayload {
  session_id: string;
  code: number;
}

export interface PtyErrorPayload {
  session_id: string;
  error_type: string;
  message: string;
}

export interface AiStreamChunkPayload {
  request_id: string;
  text: string;
}

export interface AiStreamEndPayload {
  request_id: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  stop_reason: string;
}

export interface AiErrorPayload {
  request_id: string;
  error_type: string;
  message: string;
}

export function onPtyData(
  handler: (payload: PtyDataPayload) => void
): Promise<UnlistenFn> {
  return listen<PtyDataPayload>("pty:data", (event) => handler(event.payload));
}

export function onPtyExit(
  handler: (payload: PtyExitPayload) => void
): Promise<UnlistenFn> {
  return listen<PtyExitPayload>("pty:exit", (event) => handler(event.payload));
}

export function onPtyError(
  handler: (payload: PtyErrorPayload) => void
): Promise<UnlistenFn> {
  return listen<PtyErrorPayload>("pty:error", (event) =>
    handler(event.payload)
  );
}

export function onAiStreamChunk(
  handler: (payload: AiStreamChunkPayload) => void
): Promise<UnlistenFn> {
  return listen<AiStreamChunkPayload>("ai:stream-chunk", (event) =>
    handler(event.payload)
  );
}

export function onAiStreamEnd(
  handler: (payload: AiStreamEndPayload) => void
): Promise<UnlistenFn> {
  return listen<AiStreamEndPayload>("ai:stream-end", (event) =>
    handler(event.payload)
  );
}

export function onAiError(
  handler: (payload: AiErrorPayload) => void
): Promise<UnlistenFn> {
  return listen<AiErrorPayload>("ai:error", (event) =>
    handler(event.payload)
  );
}
