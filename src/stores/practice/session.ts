import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { PracticeMode, SessionDetail } from '../../types/practice';

interface SessionState {
  currentSessionId: string | null;
  sessionResult: SessionDetail | null;
  loading: boolean;
  mode: PracticeMode | null;
  currentSentenceIndex: number;
}

export const useSessionStore = defineStore('practiceSession', {
  state: (): SessionState => ({
    currentSessionId: null,
    sessionResult: null,
    loading: false,
    mode: null,
    currentSentenceIndex: 0,
  }),

  actions: {
    async startSession(exerciseId: string, mode: PracticeMode): Promise<string> {
      this.loading = true;
      this.mode = mode;
      this.currentSentenceIndex = 0;
      try {
        const sessionId = await invoke<string>('start_session', {
          exerciseId,
          mode,
        });
        this.currentSessionId = sessionId;
        return sessionId;
      } finally {
        this.loading = false;
      }
    },

    async endSession(): Promise<number> {
      if (!this.currentSessionId) return 0;
      try {
        const score = await invoke<number>('end_session', {
          sessionId: this.currentSessionId,
        });
        return score;
      } finally {
        this.currentSessionId = null;
      }
    },

    async loadSession(id: string): Promise<void> {
      this.loading = true;
      try {
        this.sessionResult = await invoke<SessionDetail>('get_session', { id });
      } finally {
        this.loading = false;
      }
    },

    async recordAttempt(sentenceId: string, audioData: number[], durationMs: number) {
      if (!this.currentSessionId) throw new Error('No active session');
      return invoke('record_attempt', {
        sessionId: this.currentSessionId,
        sentenceId,
        audioData,
        durationMs,
      });
    },

    nextSentence(): void {
      this.currentSentenceIndex += 1;
    },

    prevSentence(): void {
      if (this.currentSentenceIndex > 0) {
        this.currentSentenceIndex -= 1;
      }
    },

    resetSession(): void {
      this.currentSessionId = null;
      this.sessionResult = null;
      this.currentSentenceIndex = 0;
      this.mode = null;
    },
  },
});
