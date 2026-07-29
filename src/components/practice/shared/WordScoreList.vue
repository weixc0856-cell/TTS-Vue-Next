<script setup lang="ts">
import type { WordStatus } from '../../../types/practice';

defineProps<{
  wordScores: { word: string; status: WordStatus; confidence: number }[];
}>();

function statusColor(status: WordStatus): string {
  switch (status) {
    case 'correct': return 'rgba(34, 197, 94, 0.15)';
    case 'wrong': return 'rgba(239, 68, 68, 0.15)';
    case 'omitted': return 'rgba(148, 163, 184, 0.15)';
    case 'extra': return 'rgba(255, 159, 67, 0.2)';
  }
}

function statusBorder(status: WordStatus): string {
  switch (status) {
    case 'correct': return 'rgba(34, 197, 94, 0.3)';
    case 'wrong': return 'rgba(239, 68, 68, 0.3)';
    case 'omitted': return 'rgba(148, 163, 184, 0.3)';
    case 'extra': return 'rgba(255, 159, 67, 0.4)';
  }
}

function statusLabel(status: WordStatus): string {
  switch (status) {
    case 'correct': return 'correct';
    case 'wrong': return 'wrong';
    case 'omitted': return 'omitted';
    case 'extra': return 'extra';
  }
}
</script>

<template>
  <div class="word-score-list">
    <div class="word-scores-label text-caption text-medium-emphasis mb-1">
      Word-by-word breakdown
    </div>
    <div class="word-tags">
      <span
        v-for="(ws, i) in wordScores"
        :key="i"
        class="word-tag"
        :title="statusLabel(ws.status)"
        :style="{
          background: statusColor(ws.status),
          border: `1px solid ${statusBorder(ws.status)}`,
        }">
        {{ ws.word }}
      </span>
      <span v-if="wordScores.length === 0" class="text-caption text-medium-emphasis">
        No results
      </span>
    </div>
  </div>
</template>

<style scoped>
.word-scores-label {
  user-select: none;
}

.word-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.word-tag {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 0.95em;
  line-height: 1.5;
}
</style>
