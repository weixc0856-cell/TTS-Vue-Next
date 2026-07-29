<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import ScoreCard from '../../components/practice/shared/ScoreCard.vue';
import WordScoreList from '../../components/practice/shared/WordScoreList.vue';
import type { PracticeSession, Attempt, ScoreResult } from '../../types/practice';

interface SessionDetail {
  session: PracticeSession;
  attempts: Attempt[];
}

const route = useRoute();
const router = useRouter();
const { locale } = useI18n();

const detail = ref<SessionDetail | null>(null);
const loading = ref(true);

onMounted(async () => {
  const id = route.params.id as string;
  loading.value = true;
  try {
    detail.value = await invoke<SessionDetail>('get_session', { id });
  } catch (e) {
    console.error('Failed to load session:', e);
  } finally {
    loading.value = false;
  }
});

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString(locale.value === 'zh' ? 'zh-CN' : 'en-US', {
      weekday: 'short', year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit',
    });
  } catch {
    return dateStr;
  }
}

function getScoreFromAttempt(a: Attempt): ScoreResult | null {
  if (a.score === null || a.score === undefined) return null;
  return {
    overall: a.score,
    accuracy: a.wordScores.length > 0 ? (a.wordScores.filter(w => w.status === 'correct').length / a.wordScores.length) * 100 : 0,
    completeness: a.completeness || 0,
    wordScores: a.wordScores,
  };
}
</script>

<template>
  <v-container fluid class="detail-page">
    <div class="mb-3">
      <v-btn variant="text" size="small" prepend-icon="mdi-arrow-left" @click="router.push('/practice/history')">
        Back
      </v-btn>
    </div>

    <div v-if="loading" class="text-center py-8">
      <v-progress-circular indeterminate size="28" />
    </div>

    <template v-else-if="detail">
      <div class="detail-header mb-4">
        <h2 class="text-h5 font-weight-bold">Session Detail</h2>
        <p class="text-body-2 text-medium-emphasis">
          {{ detail.session.mode }} · {{ formatDate(detail.session.startedAt) }}
        </p>
      </div>

      <div v-if="detail.attempts.length === 0" class="text-center py-8 text-medium-emphasis">
        No attempts recorded in this session.
      </div>

      <div v-else class="attempts-list">
        <v-card
          v-for="a in detail.attempts"
          :key="a.id"
          flat
          class="attempt-card glass-panel mb-3">
          <v-card-text>
            <div class="d-flex justify-space-between align-start mb-2">
              <div>
                <div class="text-body-2 font-weight-medium">Attempt</div>
                <div v-if="a.transcript" class="text-caption text-medium-emphasis mt-1">
                  Transcript: {{ a.transcript }}
                </div>
              </div>
              <v-chip v-if="a.score !== null && a.score !== undefined" size="small" :color="a.score >= 80 ? 'success' : a.score >= 60 ? 'warning' : 'error'" variant="tonal">
                {{ Math.round(a.score) }}/100
              </v-chip>
            </div>

            <ScoreCard v-if="a.score !== null" :score="getScoreFromAttempt(a)!" class="mb-2" />
            <WordScoreList v-if="a.wordScores.length > 0" :word-scores="a.wordScores" />

            <div class="text-caption text-medium-emphasis mt-2">
              Duration: {{ a.durationMs ? `${(a.durationMs / 1000).toFixed(1)}s` : '—' }}
            </div>
          </v-card-text>
        </v-card>
      </div>
    </template>
  </v-container>
</template>

<style scoped>
.detail-page {
  padding: 10px;
  height: 100%;
  overflow-y: auto;
  background:
    radial-gradient(circle at top right, rgba(var(--v-theme-primary), 0.08), transparent 22%),
    linear-gradient(180deg, rgba(var(--v-theme-surface), 1), rgba(var(--v-theme-surface), 0.98));
}

.detail-header {
  text-align: center;
}

.attempt-card {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}
</style>
