import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type {
  Exercise,
  PracticeMode,
  ScenarioCategory,
  Difficulty,
} from '../../types/practice';

interface ContentState {
  exercises: Exercise[];
  currentExercise: Exercise | null;
  loading: boolean;
  filter: {
    mode: PracticeMode | 'all';
    category: ScenarioCategory | 'all';
    difficulty: Difficulty | 'all';
    searchQuery: string;
  };
}

export const useContentStore = defineStore('practiceContent', {
  state: (): ContentState => ({
    exercises: [],
    currentExercise: null,
    loading: false,
    filter: {
      mode: 'all',
      category: 'all',
      difficulty: 'all',
      searchQuery: '',
    },
  }),

  getters: {
    filteredExercises(state): Exercise[] {
      let result = state.exercises;

      if (state.filter.mode !== 'all') {
        result = result.filter((e) => e.mode === state.filter.mode);
      }
      if (state.filter.category !== 'all') {
        result = result.filter((e) => e.category === state.filter.category);
      }
      if (state.filter.difficulty !== 'all') {
        result = result.filter((e) => e.difficulty === state.filter.difficulty);
      }
      if (state.filter.searchQuery) {
        const q = state.filter.searchQuery.toLowerCase();
        result = result.filter(
          (e) =>
            e.title.toLowerCase().includes(q) ||
            e.sentences.some((s) => s.text.toLowerCase().includes(q)),
        );
      }

      return result;
    },

    shadowingExercises(): Exercise[] {
      return this.filteredExercises.filter((e) => e.mode === 'shadowing');
    },

    roleplayExercises(): Exercise[] {
      return this.filteredExercises.filter((e) => e.mode === 'roleplay');
    },
  },

  actions: {
    async listExercises(
      mode?: string,
      category?: string,
      difficulty?: string,
    ): Promise<void> {
      this.loading = true;
      try {
        this.exercises = await invoke<Exercise[]>('list_exercises', {
          mode: mode || null,
          category: category || null,
          difficulty: difficulty || null,
        });
      } finally {
        this.loading = false;
      }
    },

    async getExerciseDetail(id: string): Promise<void> {
      this.loading = true;
      try {
        this.currentExercise = await invoke<Exercise>('get_exercise_detail', {
          id,
        });
      } finally {
        this.loading = false;
      }
    },

    async seedContent(): Promise<void> {
      try {
        await invoke<string[]>('seed_content');
        await this.listExercises();
      } catch (e) {
        console.error('Failed to seed content:', e);
      }
    },

    setFilter(partial: Partial<ContentState['filter']>): void {
      this.filter = { ...this.filter, ...partial };
    },

    async createCustomExercise(
      title: string,
      sentences: string[],
      translations: (string | null)[],
    ): Promise<Exercise> {
      const exercise = await invoke<Exercise>('create_custom_exercise', {
        title,
        sentences,
        translations,
      });
      this.exercises.push(exercise);
      return exercise;
    },

    clearCurrent(): void {
      this.currentExercise = null;
    },
  },
});
