import { beforeEach, describe, expect, test, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useContentStore } from './content';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

const mockExercises = [
  {
    id: 'ex-1',
    title: 'Coffee Shop',
    mode: 'shadowing' as const,
    difficulty: 'beginner' as const,
    category: 'daily' as const,
    source: 'builtin' as const,
    sentences: [{ id: 's1', text: 'Hello', orderIndex: 0 }],
    metadata: {},
  },
  {
    id: 'ex-2',
    title: 'Hotel Check-in',
    mode: 'roleplay' as const,
    difficulty: 'intermediate' as const,
    category: 'travel' as const,
    source: 'builtin' as const,
    sentences: [{ id: 's2', text: 'Welcome', orderIndex: 0 }],
    metadata: {},
  },
];

describe('useContentStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  test('listExercises loads exercises', async () => {
    invokeMock.mockResolvedValue(mockExercises);

    const store = useContentStore();
    await store.listExercises();

    expect(store.exercises).toHaveLength(2);
    expect(store.exercises[0].title).toBe('Coffee Shop');
  });

  test('getExerciseDetail loads single exercise', async () => {
    invokeMock.mockResolvedValue(mockExercises[0]);

    const store = useContentStore();
    await store.getExerciseDetail('ex-1');

    expect(store.currentExercise?.title).toBe('Coffee Shop');
    expect(store.currentExercise?.mode).toBe('shadowing');
  });

  test('seedContent calls listExercises after seeding', async () => {
    invokeMock.mockResolvedValueOnce(['ex-1']);
    invokeMock.mockResolvedValueOnce(mockExercises);

    const store = useContentStore();
    await store.seedContent();

    expect(invokeMock).toHaveBeenCalledWith('seed_content');
    expect(invokeMock).toHaveBeenCalledWith('list_exercises', expect.any(Object));
  });

  test('shadowingExercises filters correctly', () => {
    const store = useContentStore();
    store.exercises = [...mockExercises];

    const shadowing = store.shadowingExercises;
    expect(shadowing).toHaveLength(1);
    expect(shadowing[0].mode).toBe('shadowing');
  });

  test('filteredExercises applies search query', () => {
    const store = useContentStore();
    store.exercises = [...mockExercises];
    store.filter.searchQuery = 'Coffee';

    const filtered = store.filteredExercises;
    expect(filtered).toHaveLength(1);
    expect(filtered[0].title).toBe('Coffee Shop');
  });

  test('filteredExercises applies mode filter', () => {
    const store = useContentStore();
    store.exercises = [...mockExercises];
    store.filter.mode = 'roleplay';

    const filtered = store.filteredExercises;
    expect(filtered).toHaveLength(1);
    expect(filtered[0].mode).toBe('roleplay');
  });

  test('setFilter updates partial filter', () => {
    const store = useContentStore();
    store.setFilter({ category: 'travel', mode: 'roleplay' });

    expect(store.filter.category).toBe('travel');
    expect(store.filter.mode).toBe('roleplay');
    expect(store.filter.difficulty).toBe('all'); // unchanged
  });

  test('createCustomExercise adds to list', async () => {
    const customExercise = {
      id: 'custom-123',
      title: 'My Custom',
      mode: 'shadowing',
      difficulty: 'beginner',
      category: 'daily',
      source: 'user',
      sentences: [{ id: 'c-s0', text: 'Test sentence', orderIndex: 0 }],
      metadata: {},
    };
    invokeMock.mockResolvedValue(customExercise);

    const store = useContentStore();
    const result = await store.createCustomExercise('My Custom', ['Test sentence'], [null]);

    expect(store.exercises).toHaveLength(1);
    expect(result.id).toBe('custom-123');
  });

  test('clearCurrent resets currentExercise', () => {
    const store = useContentStore();
    store.currentExercise = mockExercises[0];
    store.clearCurrent();
    expect(store.currentExercise).toBeNull();
  });
});
