<script setup lang="ts">
import { computed } from 'vue';
import type { ScoreResult } from '../../../types/practice';

const props = defineProps<{
  score: ScoreResult | null;
}>();

const colorClass = computed(() => {
  if (!props.score) return 'text-medium-emphasis';
  const s = props.score.overall;
  if (s >= 80) return 'text-success';
  if (s >= 60) return 'text-warning';
  return 'text-error';
});

const ringColor = computed(() => {
  if (!props.score) return 'grey';
  const s = props.score.overall;
  if (s >= 80) return '#22c55e';
  if (s >= 60) return '#ff9f43';
  return '#ef4444';
});
</script>

<template>
  <v-card flat class="score-card glass-panel">
    <v-card-text class="score-card__body">
      <div v-if="!score" class="score-card__empty text-caption text-medium-emphasis">
        {{ $t('practice.scoring.noScore') }}
      </div>

      <template v-else>
        <div class="score-card__overall">
          <div class="score-ring">
            <svg viewBox="0 0 64 64" class="score-ring-svg">
              <circle cx="32" cy="32" r="28" fill="none" stroke="rgba(var(--v-theme-on-surface), 0.08)" stroke-width="4" />
              <circle
                cx="32"
                cy="32"
                r="28"
                fill="none"
                :stroke="ringColor"
                stroke-width="4"
                stroke-linecap="round"
                :stroke-dasharray="2 * Math.PI * 28"
                :stroke-dashoffset="2 * Math.PI * 28 * (1 - score.overall / 100)"
                transform="rotate(-90, 32, 32)" />
            </svg>
            <div class="score-ring-value" :class="colorClass">
              {{ Math.round(score.overall) }}
            </div>
          </div>
        </div>

        <div class="score-card__details">
          <div class="score-detail">
            <span class="text-caption text-medium-emphasis">{{ $t('practice.scoring.accuracy') }}</span>
            <v-progress-linear
              :model-value="score.accuracy"
              color="primary"
              height="6"
              rounded
              class="mt-1" />
          </div>
          <div class="score-detail">
            <span class="text-caption text-medium-emphasis">{{ $t('practice.scoring.completeness') }}</span>
            <v-progress-linear
              :model-value="score.completeness"
              color="success"
              height="6"
              rounded
              class="mt-1" />
          </div>
        </div>
      </template>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.score-card {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}

.score-card__body {
  display: flex;
  gap: 20px;
  align-items: center;
}

.score-card__empty {
  width: 100%;
  text-align: center;
  padding: 12px 0;
}

.score-card__overall {
  flex-shrink: 0;
}

.score-ring {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.score-ring-svg {
  width: 80px;
  height: 80px;
}

.score-ring-value {
  position: absolute;
  font-size: 24px;
  font-weight: 700;
}

.score-card__details {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
