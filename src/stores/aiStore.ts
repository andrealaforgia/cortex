import { create } from "zustand";
import type { AIMessage, AIConversation } from "../lib/types";

interface AIState {
  isStreaming: boolean;
  currentRequestId: string | null;
  streamingText: string;
  conversations: AIConversation[];
  activeConversationId: string | null;
  error: string | null;
  setStreaming: (isStreaming: boolean, requestId?: string | null) => void;
  appendStreamText: (text: string) => void;
  clearStreamText: () => void;
  addMessage: (conversationId: string, message: AIMessage) => void;
  setActiveConversation: (id: string | null) => void;
  setError: (error: string | null) => void;
}

export const useAIStore = create<AIState>((set) => ({
  isStreaming: false,
  currentRequestId: null,
  streamingText: "",
  conversations: [],
  activeConversationId: null,
  error: null,
  setStreaming: (isStreaming, requestId = null) =>
    set({ isStreaming, currentRequestId: requestId }),
  appendStreamText: (text) =>
    set((state) => ({ streamingText: state.streamingText + text })),
  clearStreamText: () => set({ streamingText: "" }),
  addMessage: (conversationId, message) =>
    set((state) => ({
      conversations: state.conversations.map((c) =>
        c.id === conversationId
          ? {
              ...c,
              messages: [...c.messages, message],
              updatedAt: new Date().toISOString(),
            }
          : c
      ),
    })),
  setActiveConversation: (id) => set({ activeConversationId: id }),
  setError: (error) => set({ error }),
}));
