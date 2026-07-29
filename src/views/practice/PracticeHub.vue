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
    <div class="practice-hub__header mb-4">
      <div class="d-flex align-center justify-center ga-3">
        <h2 class="text-h5 font-weight-bold">{{ $t('practice.hub.title') }}</h2>
        <v-btn
          variant="text"
          size="small"
          color="primary"
          prepend-icon="mdi-history"
          @click="router.push('/practice/history')">
          History
        </v-btn>
      </div>
      <p class="text-body-2 text-medium-emphasis mt-1">
        {{ $t('practice.hub.subtitle') }}
      </p>
    </div>

    <!-- Whisper setup notice -->
    <v-alert
      v-if="!whisperConfigured"
      type="warning"
      density="compact"
      variant="tonal"
      class="mb-4 mx-auto"
      style="max-width: 600px"
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

    <div class="practice-modes">
      <!-- Shadowing -->
      <v-card flat class="mode-card glass-panel" @click="startShadowing">
        <v-card-text class="mode-card__body">
          <v-avatar color="success" variant="tonal" size="52" class="mode-icon">
            <v-icon size="28">mdi-headphones</v-icon>
          </v-avatar>
          <div class="mode-info">
            <div class="text-subtitle-1 font-weight-medium">
              {{ $t('practice.hub.shadowing') }}
            </div>
            <div class="text-caption text-medium-emphasis mt-1">
              {{ $t('practice.hub.shadowingDesc') }}
            </div>
          </div>
          <v-icon color="primary" size="24">mdi-arrow-right</v-icon>
        </v-card-text>
      </v-card>

      <!-- Role-play -->
      <v-card flat class="mode-card glass-panel" @click="startRoleplay">
        <v-card-text class="mode-card__body">
          <v-avatar color="primary" variant="tonal" size="52" class="mode-icon">
            <v-icon size="28">mdi-chat-dots-outline</v-icon>
          </v-avatar>
          <div class="mode-info">
            <div class="text-subtitle-1 font-weight-medium">
              {{ $t('practice.hub.roleplay') }}
            </div>
            <div class="text-caption text-medium-emphasis mt-1">
              {{ $t('practice.hub.roleplayDesc') }}
            </div>
          </div>
          <v-icon color="primary" size="24">mdi-arrow-right</v-icon>
        </v-card-text>
      </v-card>

      <!-- Pronunciation -->
      <v-card flat class="mode-card glass-panel" @click="startPronunciation">
        <v-card-text class="mode-card__body">
          <v-avatar color="warning" variant="tonal" size="52" class="mode-icon">
            <v-icon size="28">mdi-alphabetical-variant</v-icon>
          </v-avatar>
          <div class="mode-info">
            <div class="text-subtitle-1 font-weight-medium">
              {{ $t('practice.hub.pronunciation') }}
            </div>
            <div class="text-caption text-medium-emphasis mt-1">
              {{ $t('practice.hub.pronunciationDesc') }}
            </div>
          </div>
          <v-icon color="primary" size="24">mdi-arrow-right</v-icon>
        </v-card-text>
      </v-card>

      <!-- Free Talk (disabled) -->
      <v-card flat class="mode-card glass-panel mode-card--disabled">
        <v-card-text class="mode-card__body">
          <v-avatar color="secondary" variant="tonal" size="52" class="mode-icon">
            <v-icon size="28">mdi-robot-outline</v-icon>
          </v-avatar>
          <div class="mode-info">
            <div class="text-subtitle-1 font-weight-medium">
              {{ $t('practice.hub.freetalk') }}
            </div>
            <div class="text-caption text-medium-emphasis mt-1">
              {{ $t('practice.hub.freetalkDesc') }}
            </div>
          </div>
          <v-chip size="x-small" color="secondary" variant="tonal">Soon</v-chip>
        </v-card-text>
      </v-card>
    </div>
  </v-container>
</template>

<style scoped>
.practice-hub {
  padding: 10px;
  height: 100%;
  background:
    radial-gradient(
      circle at top right,
      rgba(var(--v-theme-primary), 0.08),
      transparent 22%
    ),
    linear-gradient(
      180deg,
      rgba(var(--v-theme-surface), 1),
      rgba(var(--v-theme-surface), 0.98)
    );
}

.practice-hub__header {
  text-align: center;
  padding: 12px 0 8px;
}

.practice-modes {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
  max-width: 800px;
  margin: 0 auto;
}

.mode-card {
  cursor: pointer;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background:
    radial-gradient(
      circle at top right,
      rgba(var(--v-theme-primary), 0.08),
      transparent 30%
    ),
    rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
  transition: border-color 0.15s ease, transform 0.15s ease;
}

.mode-card:hover {
  border-color: rgba(var(--v-theme-primary), 0.3);
  transform: translateY(-1px);
}

.mode-card--disabled {
  opacity: 0.6;
  cursor: default;
}

.mode-card--disabled:hover {
  border-color: rgba(var(--v-border-color), var(--v-border-opacity));
  transform: none;
}

.mode-card__body {
  display: flex;
  align-items: center;
  gap: 14px;
}

.mode-icon {
  flex-shrink: 0;
}

.mode-info {
  flex: 1;
  min-width: 0;
}
</style>
