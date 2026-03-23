import { create } from "zustand";
import type { Block } from "../lib/types";

interface TerminalState {
  sessionId: string | null;
  blocks: Block[];
  error: string | null;
  setSessionId: (id: string | null) => void;
  addBlock: (block: Block) => void;
  updateBlock: (id: string, updates: Partial<Block>) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

export const useTerminalStore = create<TerminalState>((set) => ({
  sessionId: null,
  blocks: [],
  error: null,
  setSessionId: (id) => set({ sessionId: id }),
  addBlock: (block) =>
    set((state) => ({ blocks: [...state.blocks, block] })),
  updateBlock: (id, updates) =>
    set((state) => ({
      blocks: state.blocks.map((b) =>
        b.id === id ? { ...b, ...updates } : b
      ),
    })),
  setError: (error) => set({ error }),
  reset: () => set({ sessionId: null, blocks: [], error: null }),
}));
