<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';

interface SessionSummary {
  id: string;
  exercise_title: string;
  mode: string;
  started_at: string;
  completed_at: string | null;
  overall_score: number | null;
  sentence_count: number;
  attempt_count: number;
}

interface PracticeStats {
  total_sessions: number;
  total_attempts: number;
  average_score: number | null;
  total_practice_minutes: number;
  best_score: number | null;
  recent_scores: number[];
}

const router = useRouter();
const { t, locale } = useI18n();

const sessions = ref<SessionSummary[]>([]);
const stats = ref<PracticeStats | null>(null);
const loading = ref(true);
const limit = ref(50);
const offset = ref(0);

onMounted(async () => {
  loading.value = true;
  try {
    const [sessionData, statsData] = await Promise.all([
      invoke<SessionSummary[]>('get_session_history', { limit: limit.value, offset: offset.value }),
      invoke<PracticeStats>('get_practice_stats'),
    ]);
    sessions.value = sessionData;
    stats.value = statsData;
  } catch (e) {
    console.error('Failed to load history:', e);
  } finally {
    loading.value = false;
  }
});

function viewSession(id: string) {
  router.push(`/practice/history/${id}`);
}

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString(locale.value === 'zh' ? 'zh-CN' : 'en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return dateStr;
  }
}

function modeLabel(mode: string): string {
  if (mode === 'shadowing') return t('practice.hub.shadowing');
  if (mode === 'roleplay') return t('practice.hub.roleplay');
  return mode;
}

function scoreColor(score: number | null): string {
  if (score === null) return 'grey';
  if (score >= 80) return 'success';
  if (score >= 60) return 'warning';
  return 'error';
}
</script>

<template>
  <v-container fluid class="history-page">
    <div class="history-header mb-4">
      <h2 class="text-h5 font-weight-bold">Practice History</h2>
      <p class="text-body-2 text-medium-emphasis">Track your progress over time</p>
    </div>

    <!-- Stats Overview -->
    <v-row v-if="stats" class="mb-4" dense>
      <v-col cols="6" sm="3">
        <v-card flat class="stat-card glass-panel pa-3 text-center">
          <div class="text-h5 font-weight-bold">{{ stats.total_sessions }}</div>
          <div class="text-caption text-medium-emphasis">Sessions</div>
        </v-card>
      </v-col>
      <v-col cols="6" sm="3">
        <v-card flat class="stat-card glass-panel pa-3 text-center">
          <div class="text-h5 font-weight-bold">{{ stats.total_attempts }}</div>
          <div class="text-caption text-medium-emphasis">Attempts</div>
        </v-card>
      </v-col>
      <v-col cols="6" sm="3">
        <v-card flat class="stat-card glass-panel pa-3 text-center">
          <div class="text-h5 font-weight-bold" :class="stats.average_score !== null ? scoreColor(stats.average_score) : ''">
            {{ stats.average_score !== null ? Math.round(stats.average_score) : '—' }}
          </div>
          <div class="text-caption text-medium-emphasis">Avg Score</div>
        </v-card>
      </v-col>
      <v-col cols="6" sm="3">
        <v-card flat class="stat-card glass-panel pa-3 text-center">
          <div class="text-h5 font-weight-bold">
            {{ Math.round(stats.total_practice_minutes) }}
          </div>
          <div class="text-caption text-medium-emphasis">Minutes</div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Score Trend -->
    <v-card v-if="stats && stats.recent_scores.length > 0" flat class="trend-card glass-panel pa-4 mb-4">
      <div class="text-subtitle-2 mb-2">Recent Scores</div>
      <div class="trend-bars">
        <div
          v-for="(score, i) in [...stats.recent_scores].reverse()"
          :key="i"
          class="trend-bar-wrapper">
          <div
            class="trend-bar"
            :style="{
              height: `${score}%`,
              background: score >= 80 ? 'rgb(var(--v-theme-success))' : score >= 60 ? 'rgb(var(--v-theme-warning))' : 'rgb(var(--v-theme-error))',
            }" />
          <div class="text-caption text-center mt-1">{{ Math.round(score) }}</div>
        </div>
      </div>
    </v-card>

    <!-- Session List -->
    <div v-if="loading" class="text-center py-8">
      <v-progress-circular indeterminate size="28" />
    </div>

    <div v-else-if="sessions.length === 0" class="text-center py-8 text-medium-emphasis">
      <v-icon size="48" class="mb-2">mdi-history</v-icon>
      <p>No practice sessions yet. Start practicing!</p>
    </div>

    <div v-else class="session-list">
      <v-card
        v-for="s in sessions"
        :key="s.id"
        flat
        class="session-card glass-panel"
        @click="viewSession(s.id)">
        <v-card-text class="d-flex justify-space-between align-center">
          <div class="session-info">
            <div class="text-subtitle-2 font-weight-medium">{{ s.exercise_title }}</div>
            <div class="text-caption text-medium-emphasis">
              {{ modeLabel(s.mode) }} · {{ formatDate(s.started_at) }}
            </div>
          </div>
          <div class="d-flex align-center ga-3">
            <v-chip size="x-small" variant="tonal" color="primary">
              {{ s.sentence_count }} sentences
            </v-chip>
            <div
              v-if="s.overall_score !== null"
              class="text-h6 font-weight-bold"
              :class="scoreColor(s.overall_score)">
              {{ Math.round(s.overall_score) }}
            </div>
          </div>
        </v-card-text>
      </v-card>
    </div>
  </v-container>
</template>

<style scoped>
.history-page {
  padding: 10px;
  height: 100%;
  overflow-y: auto;
  background:
    radial-gradient(circle at top right, rgba(var(--v-theme-primary), 0.08), transparent 22%),
    linear-gradient(180deg, rgba(var(--v-theme-surface), 1), rgba(var(--v-theme-surface), 0.98));
}

.history-header {
  text-align: center;
}

.stat-card {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}

.trend-card {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}

.trend-bars {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  height: 100px;
}

.trend-bar-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
  justify-content: flex-end;
}

.trend-bar {
  width: 100%;
  min-height: 4px;
  border-radius: 4px 4px 0 0;
  transition: height 0.3s ease;
}

.session-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.session-card {
  cursor: pointer;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background: rgba(var(--v-theme-surface), 0.7);
  backdrop-filter: blur(12px);
  transition: border-color 0.15s ease;
}

.session-card:hover {
  border-color: rgba(var(--v-theme-primary), 0.3);
}
</style>
