import { beforeEach, describe, expect, test } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useScoringStore } from './scoring';
import type { ScoreResult } from '../../types/practice';

const mockScore: ScoreResult = {
  overall: 85,
  accuracy: 80,
  completeness: 90,
  wordScores: [
    { word: 'hello', status: 'correct', confidence: 0.95 },
    { word: 'world', status: 'correct', confidence: 0.9 },
  ],
};

describe('useScoringStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  test('setScore stores score and adds to history', () => {
    const store = useScoringStore();
    store.setScore(mockScore);

    expect(store.lastScore).toEqual(mockScore);
    expect(store.scoreHistory).toHaveLength(1);
  });

  test('clearScore clears lastScore', () => {
    const store = useScoringStore();
    store.setScore(mockScore);
    store.clearScore();

    expect(store.lastScore).toBeNull();
    expect(store.scoreHistory).toHaveLength(1); // history preserved
  });

  test('clearHistory clears everything', () => {
    const store = useScoringStore();
    store.setScore(mockScore);
    store.clearHistory();

    expect(store.lastScore).toBeNull();
    expect(store.scoreHistory).toHaveLength(0);
  });

  test('multiple scores accumulate in history', () => {
    const store = useScoringStore();
    store.setScore(mockScore);
    store.setScore({ ...mockScore, overall: 92 });

    expect(store.scoreHistory).toHaveLength(2);
    expect(store.lastScore?.overall).toBe(92);
  });
});
