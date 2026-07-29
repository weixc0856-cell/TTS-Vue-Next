<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useContentStore } from '../../../stores/practice/content';
import type { Exercise, PracticeMode } from '../../../types/practice';

const content = useContentStore();

const props = withDefaults(
  defineProps<{
    mode?: PracticeMode;
    title?: string;
  }>(),
  { mode: undefined, title: undefined },
);

const emit = defineEmits<{
  select: [exercise: Exercise];
}>();

const filtered = computed(() => {
  if (props.mode) {
    return content.filteredExercises.filter((e) => e.mode === props.mode);
  }
  return content.filteredExercises;
});

const categories = computed(() => {
  const cats = new Set(content.exercises.map((e) => e.category));
  return Array.from(cats);
});

const difficulties = computed(() => {
  const diffs = new Set(content.exercises.map((e) => e.difficulty));
  return Array.from(diffs);
});

function selectExercise(exercise: Exercise) {
  emit('select', exercise);
}

onMounted(() => {
  if (content.exercises.length === 0) {
    content.seedContent();
  }
});
</script>

<template>
  <div class="scenario-selector">
    <div v-if="title" class="text-h6 mb-3">{{ title }}</div>

    <!-- Filters -->
    <div class="d-flex ga-2 mb-3 flex-wrap">
      <v-select
        v-model="content.filter.category"
        :items="['all', ...categories]"
        density="compact"
        variant="outlined"
        hide-details
        style="min-width: 120px; max-width: 160px" />
      <v-select
        v-model="content.filter.difficulty"
        :items="['all', ...difficulties]"
        density="compact"
        variant="outlined"
        hide-details
        style="min-width: 140px; max-width: 180px" />
    </div>

    <!-- Loading -->
    <div v-if="content.loading" class="text-center py-6">
      <v-progress-circular indeterminate size="28" />
    </div>

    <!-- Exercise List -->
    <div v-else-if="filtered.length === 0" class="text-center py-6 text-medium-emphasis text-body-2">
      No exercises found
    </div>

    <div v-else class="scenario-list">
      <v-card
        v-for="exercise in filtered"
        :key="exercise.id"
        flat
        class="scenario-card glass-panel"
        @click="selectExercise(exercise)">
        <v-card-text>
          <div class="d-flex justify-space-between align-start">
            <div>
              <div class="text-subtitle-2 font-weight-medium">{{ exercise.title }}</div>
              <div class="text-caption text-medium-emphasis mt-1">
                {{ exercise.difficulty }} · {{ exercise.category }}
                · {{ exercise.sentences.length }} sentences
              </div>
            </div>
            <v-chip size="x-small" variant="tonal" color="primary">
              {{ exercise.mode }}
            </v-chip>
          </div>
        </v-card-text>
      </v-card>
    </div>
  </div>
</template>

<style scoped>
.scenario-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
}

.scenario-card {
  cursor: pointer;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgba(var(--v-theme-surface), 0.7);
  backdrop-filter: blur(12px);
  transition: border-color 0.15s ease;
}

.scenario-card:hover {
  border-color: rgba(var(--v-theme-primary), 0.4);
}
</style>
