<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useSettingsStore } from '../../stores/settings';
import { useContentStore } from '../../stores/practice/content';

const router = useRouter();
const settingsStore = useSettingsStore();
const content = useContentStore();

const whisperConfigured = computed(() => {
  return !!(settingsStore.whisperBinPath && settingsStore.whisperModelPath);
});

onMounted(() => {
  content.seedContent();
});

function startShadowing() {
  router.push('/practice/shadowing');
}

function startRoleplay() {
  router.push('/practice/roleplay');
}

function startPronunciation() {
  router.push('/practice/pronunciation');
}
</script>

<template>
  <v-container fluid class="practice-hub">
    <!-- Header -->
    <div class="hub-header">
      <div class="d-flex align-center justify-center ga-3">
        <h2 class="text-h4 font-weight-bold hub-title">{{ $t('practice.hub.title') }}</h2>
        <v-btn
          variant="text"
          size="small"
          color="secondary"
          prepend-icon="mdi-history"
          @click="router.push('/practice/history')"
          class="history-btn">
          History
        </v-btn>
      </div>
      <p class="text-body-2 text-medium-emphasis mt-1 hub-subtitle">
        {{ $t('practice.hub.subtitle') }}
      </p>
    </div>

    <!-- Whisper notice -->
    <v-alert
      v-if="!whisperConfigured"
      type="warning"
      density="compact"
      variant="tonal"
      class="mb-5 mx-auto whisper-notice"
      closable>
      <template #title>
        Whisper not configured
      </template>
      <template #default>
        <p class="text-body-2">
          Speech recognition is not configured yet.
          <router-link to="/settings" class="text-primary font-weight-medium">Go to Settings</router-link>
          to set up whisper-cli and a model file for pronunciation scoring.
        </p>
      </template>
    </v-alert>

    <!-- Mode Cards -->
    <div class="mode-grid">
      <!-- Shadowing -->
      <v-card flat class="mode-card" @click="startShadowing">
        <v-card-text class="mode-card__body">
          <div class="mode-card__icon mode-card__icon--shadowing">
            <v-icon size="28">mdi-headphones</v-icon>
          </div>
          <div class="mode-card__info">
            <div class="mode-card__title">{{ $t('practice.hub.shadowing') }}</div>
            <div class="mode-card__desc">{{ $t('practice.hub.shadowingDesc') }}</div>
          </div>
          <div class="mode-card__action">
            <v-icon color="rgb(var(--v-theme-primary))" size="22">mdi-arrow-right</v-icon>
          </div>
        </v-card-text>
      </v-card>

      <!-- Role-play -->
      <v-card flat class="mode-card" @click="startRoleplay">
        <v-card-text class="mode-card__body">
          <div class="mode-card__icon mode-card__icon--roleplay">
            <v-icon size="28">mdi-chat-dots-outline</v-icon>
          </div>
          <div class="mode-card__info">
            <div class="mode-card__title">{{ $t('practice.hub.roleplay') }}</div>
            <div class="mode-card__desc">{{ $t('practice.hub.roleplayDesc') }}</div>
          </div>
          <div class="mode-card__action">
            <v-icon color="rgb(var(--v-theme-primary))" size="22">mdi-arrow-right</v-icon>
          </div>
        </v-card-text>
      </v-card>

      <!-- Pronunciation -->
      <v-card flat class="mode-card" @click="startPronunciation">
        <v-card-text class="mode-card__body">
          <div class="mode-card__icon mode-card__icon--pronunciation">
            <v-icon size="28">mdi-alphabetical-variant</v-icon>
          </div>
          <div class="mode-card__info">
            <div class="mode-card__title">{{ $t('practice.hub.pronunciation') }}</div>
            <div class="mode-card__desc">Minimal pairs, phonemes, and tricky sounds</div>
          </div>
          <div class="mode-card__action">
            <v-icon color="rgb(var(--v-theme-primary))" size="22">mdi-arrow-right</v-icon>
          </div>
        </v-card-text>
      </v-card>

      <!-- Free Talk (disabled) -->
      <v-card flat class="mode-card mode-card--disabled">
        <v-card-text class="mode-card__body">
          <div class="mode-card__icon mode-card__icon--freetalk">
            <v-icon size="28">mdi-robot-outline</v-icon>
          </div>
          <div class="mode-card__info">
            <div class="mode-card__title">{{ $t('practice.hub.freetalk') }}</div>
            <div class="mode-card__desc">{{ $t('practice.hub.freetalkDesc') }}</div>
          </div>
          <v-chip size="x-small" color="secondary" variant="tonal" class="mode-card__badge">Soon</v-chip>
        </v-card-text>
      </v-card>
    </div>
  </v-container>
</template>

<style scoped>
.practice-hub {
  padding: 24px;
  height: 100%;
  overflow-y: auto;
  background:
    radial-gradient(circle at 20% 10%, rgba(var(--v-theme-primary), 0.06), transparent 30%),
    radial-gradient(circle at 80% 90%, rgba(var(--v-theme-info), 0.05), transparent 28%),
    rgb(var(--v-theme-background));
}

.hub-header {
  text-align: center;
  padding: 12px 0 24px;
}

.hub-title {
  background: linear-gradient(135deg, rgb(var(--v-theme-primary)), rgb(var(--v-theme-info)));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hub-subtitle {
  max-width: 320px;
  margin: 4px auto 0;
}

.history-btn {
  opacity: 0.6;
  transition: opacity 0.15s ease;
}
.history-btn:hover { opacity: 1; }

.whisper-notice {
  max-width: 520px;
}

.mode-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
  max-width: 640px;
  margin: 0 auto;
}

.mode-card {
  cursor: pointer;
  border-radius: 16px !important;
  border: 1px solid rgba(var(--v-theme-glass-border), 0.4);
  background: rgba(var(--v-theme-glass), 0.65);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  transition: all 0.2s cubic-bezier(0.25, 0.1, 0.25, 1);
}

.mode-card:hover {
  border-color: rgba(var(--v-theme-primary), 0.35);
  background: rgba(var(--v-theme-glass), 0.8);
  box-shadow:
    0 4px 12px rgba(74, 124, 255, 0.08),
    0 8px 24px rgba(15, 23, 42, 0.06);
  transform: translateY(-2px);
}

.mode-card--disabled {
  opacity: 0.55;
  cursor: default;
}

.mode-card--disabled:hover {
  border-color: rgba(var(--v-theme-glass-border), 0.4);
  background: rgba(var(--v-theme-glass), 0.65);
  box-shadow: none;
  transform: none;
}

.mode-card__body {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px !important;
}

.mode-card__icon {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.mode-card__icon--shadowing {
  background: rgba(46, 162, 107, 0.12);
  color: rgb(var(--v-theme-success));
}

.mode-card__icon--roleplay {
  background: rgba(74, 124, 255, 0.12);
  color: rgb(var(--v-theme-primary));
}

.mode-card__icon--pronunciation {
  background: rgba(217, 154, 51, 0.12);
  color: rgb(var(--v-theme-warning));
}

.mode-card__icon--freetalk {
  background: rgba(122, 139, 165, 0.12);
  color: rgb(var(--v-theme-secondary));
}

.mode-card__info {
  flex: 1;
  min-width: 0;
}

.mode-card__title {
  font-size: 1rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: -0.01em;
}

.mode-card__desc {
  font-size: 0.8rem;
  color: rgba(var(--v-theme-on-surface), 0.55);
  margin-top: 2px;
  line-height: 1.3;
}

.mode-card__action {
  flex-shrink: 0;
  opacity: 0;
  transform: translateX(-4px);
  transition: all 0.2s ease;
}

.mode-card:hover .mode-card__action {
  opacity: 1;
  transform: translateX(0);
}

.mode-card__badge {
  flex-shrink: 0;
}
</style>
