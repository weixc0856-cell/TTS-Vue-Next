import { defineStore } from 'pinia';
import type { ScoreResult } from '../../types/practice';

interface ScoringState {
  lastScore: ScoreResult | null;
  scoreHistory: ScoreResult[];
}

export const useScoringStore = defineStore('practiceScoring', {
  state: (): ScoringState => ({
    lastScore: null,
    scoreHistory: [],
  }),

  actions: {
    setScore(score: ScoreResult): void {
      this.lastScore = score;
      this.scoreHistory.push(score);
    },

    clearScore(): void {
      this.lastScore = null;
    },

    clearHistory(): void {
      this.scoreHistory = [];
      this.lastScore = null;
    },
  },
});
