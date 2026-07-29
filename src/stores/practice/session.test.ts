import { beforeEach, describe, expect, test, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useSessionStore } from './session';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

describe('useSessionStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  test('startSession calls invoke and sets state', async () => {
    invokeMock.mockResolvedValue('session-abc');

    const store = useSessionStore();
    const sessionId = await store.startSession('ex-1', 'shadowing');

    expect(sessionId).toBe('session-abc');
    expect(store.currentSessionId).toBe('session-abc');
    expect(store.mode).toBe('shadowing');
    expect(store.currentSentenceIndex).toBe(0);
    expect(invokeMock).toHaveBeenCalledWith('start_session', {
      exerciseId: 'ex-1',
      mode: 'shadowing',
    });
  });

  test('endSession calls invoke with correct sessionId', async () => {
    invokeMock.mockResolvedValue(85);

    const store = useSessionStore();
    store.currentSessionId = 'session-abc';
    const score = await store.endSession();

    expect(score).toBe(85);
    expect(store.currentSessionId).toBeNull();
    expect(invokeMock).toHaveBeenCalledWith('end_session', {
      sessionId: 'session-abc',
    });
  });

  test('endSession returns 0 when no active session', async () => {
    const store = useSessionStore();
    const score = await store.endSession();
    expect(score).toBe(0);
  });

  test('recordAttempt calls invoke with session data', async () => {
    invokeMock.mockResolvedValue({ id: 'attempt-1' });

    const store = useSessionStore();
    store.currentSessionId = 'session-abc';
    await store.recordAttempt('sent-1', [1, 2, 3], 5000);

    expect(invokeMock).toHaveBeenCalledWith('record_attempt', {
      sessionId: 'session-abc',
      sentenceId: 'sent-1',
      audioData: [1, 2, 3],
      durationMs: 5000,
    });
  });

  test('recordAttempt throws when no active session', async () => {
    const store = useSessionStore();
    await expect(store.recordAttempt('sent-1', [1], 100)).rejects.toThrow('No active session');
  });

  test('nextSentence increments index', () => {
    const store = useSessionStore();
    store.currentSentenceIndex = 0;
    store.nextSentence();
    expect(store.currentSentenceIndex).toBe(1);
  });

  test('prevSentence decrements index', () => {
    const store = useSessionStore();
    store.currentSentenceIndex = 2;
    store.prevSentence();
    expect(store.currentSentenceIndex).toBe(1);
  });

  test('prevSentence does not go below 0', () => {
    const store = useSessionStore();
    store.currentSentenceIndex = 0;
    store.prevSentence();
    expect(store.currentSentenceIndex).toBe(0);
  });

  test('resetSession clears all state', () => {
    const store = useSessionStore();
    store.currentSessionId = 'session-abc';
    store.mode = 'shadowing';
    store.currentSentenceIndex = 3;

    store.resetSession();

    expect(store.currentSessionId).toBeNull();
    expect(store.mode).toBeNull();
    expect(store.currentSentenceIndex).toBe(0);
  });

  test('loadSession fetches session detail', async () => {
    invokeMock.mockResolvedValue({
      session: { id: 's1', exerciseId: 'ex-1', mode: 'shadowing', startedAt: '2026-01-01', overallScore: 85 },
      attempts: [{ id: 'a1', sessionId: 's1', sentenceId: 'sent-1', wordScores: [], durationMs: 1000 }],
    });

    const store = useSessionStore();
    await store.loadSession('s1');

    expect(store.sessionResult?.session.id).toBe('s1');
    expect(store.sessionResult?.attempts).toHaveLength(1);
  });
});
