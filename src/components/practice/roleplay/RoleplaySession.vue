<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useMessage } from 'vuetify-message-vue3';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../../stores/settings';
import { useSessionStore } from '../../../stores/practice/session';
import { useRecorderStore } from '../../../stores/practice/recorder';
import RecordButton from '../shared/RecordButton.vue';
import AudioVisualizer from '../shared/AudioVisualizer.vue';
import type { ScoreResult, Exercise } from '../../../types/practice';

const props = withDefaults(
  defineProps<{
    exerciseId?: string | null;
  }>(),
  { exerciseId: null },
);

const route = useRoute();
const router = useRouter();
const message = useMessage();
const settingsStore = useSettingsStore();
const session = useSessionStore();
const recorder = useRecorderStore();

interface Exchange {
  role: 'partner' | 'user';
  text: string;
  translation?: string;
  score?: ScoreResult | null;
}

const exchanges = ref<Exchange[]>([]);
const currentIndex = ref(0);
const loading = ref(true);
const partnerPlaying = ref(false);

const currentExchange = computed(() => exchanges.value[currentIndex.value] || null);
const isPartnerTurn = computed(() => currentExchange.value?.role === 'partner');
const isUserTurn = computed(() => currentExchange.value?.role === 'user');
const isComplete = computed(() => currentIndex.value >= exchanges.value.length);
const progress = computed(() => {
  if (exchanges.value.length === 0) return 0;
  return Math.round((currentIndex.value / exchanges.value.length) * 100);
});

onMounted(async () => {
  try {
    const id = props.exerciseId || route.query.exerciseId as string;

    if (id) {
      const exercise = await invoke<Exercise>('get_exercise_detail', { id });
      // Parse sentences back into exchanges
      exchanges.value = exercise.sentences.map((s) => {
        const role = s.text.startsWith('A:') || s.text.startsWith('Partner:') ? 'partner' : 'user';
        return {
          role,
          text: s.text.replace(/^(A:|B:|Partner:|You:)\s*/, ''),
          translation: s.translation,
          score: null,
        };
      });
    } else {
      // Load first roleplay exercise
      const exercises = await invoke<Exercise[]>('list_exercises', {
        mode: 'roleplay',
        category: null,
        difficulty: null,
      });
      if (exercises.length > 0) {
        const ex = exercises[0];
        exchanges.value = ex.sentences.map((s) => {
          const role = s.text.startsWith('A:') || s.text.startsWith('Partner:') ? 'partner' : 'user';
          return {
            role,
            text: s.text.replace(/^(A:|B:|Partner:|You:)\s*/, ''),
            translation: s.translation,
            score: null,
          };
        });
      }
    }

    if (exchanges.value.length > 0) {
      await session.startSession(id || 'roleplay', 'roleplay');
      // Auto-play partner's first line
      if (isPartnerTurn.value) {
        await playPartnerLine();
      }
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    loading.value = false;
  }
});

async function playPartnerLine() {
  if (!currentExchange.value) return;
  partnerPlaying.value = true;
  try {
    await invoke('tts_convert', {
      params: {
        text: currentExchange.value.text,
        voice: 'en-US-EmmaMultilingualNeural',
        rate: '+0%',
        pitch: '+0Hz',
        volume: '+0%',
        format: 'audio-24khz-48kbitrate-mono-mp3',
        task_id: `roleplay-${Date.now()}`,
        max_retries: 2,
      },
    });
  } catch (e) {
    message.error(String(e));
  } finally {
    partnerPlaying.value = false;
  }
}

async function onRecorded(audioData: Uint8Array, durationMs: number) {
  if (!currentExchange.value || !session.currentSessionId) return;

  try {
    const result = await invoke<ScoreResult>('transcribe_and_score', {
      referenceText: currentExchange.value.text,
      audioData: Array.from(audioData),
      durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });

    currentExchange.value.score = result;

    // Auto advance after a short delay
    setTimeout(() => {
      nextExchange();
    }, 1500);
  } catch (e) {
    message.error(String(e));
  }
}

function nextExchange() {
  currentIndex.value++;
  if (isPartnerTurn.value && !isComplete.value) {
    setTimeout(() => playPartnerLine(), 500);
  }
}

async function completePractice() {
  try {
    await session.endSession();
  } catch {
    // ignore
  }
  router.push('/practice');
}
</script>

<template>
  <v-container fluid class="roleplay-session">
    <div v-if="loading" class="d-flex justify-center align-center py-12">
      <v-progress-circular indeterminate size="32" />
    </div>

    <template v-else-if="isComplete">
      <div class="text-center py-8">
        <v-icon size="56" color="success" class="mb-3">mdi-check-circle</v-icon>
        <h3 class="text-h5 mb-2">{{ $t('practice.roleplay.complete') }}</h3>
        <v-btn color="primary" variant="tonal" @click="completePractice">
          {{ $t('practice.actions.backToHub') }}
        </v-btn>
      </div>
    </template>

    <template v-else>
      <div class="roleplay-workspace">
        <!-- Progress Bar -->
        <div class="d-flex align-center ga-2 mb-4">
          <v-progress-linear
            :model-value="progress"
            color="primary"
            height="4"
            rounded />
          <span class="text-caption text-medium-emphasis">{{ currentIndex }}/{{ exchanges.length }}</span>
        </div>

        <!-- Dialogue Bubbles -->
        <div class="dialogue-area">
          <div
            v-for="(ex, i) in exchanges.slice(0, currentIndex + 1)"
            :key="i"
            :class="['dialogue-bubble', ex.role === 'partner' ? 'bubble-partner' : 'bubble-user']">
            <div class="bubble-label text-caption font-weight-medium">
              {{ ex.role === 'partner' ? $t('practice.roleplay.partner') : $t('practice.roleplay.you') }}
            </div>
            <div class="bubble-text">{{ ex.text }}</div>
            <div v-if="ex.translation" class="bubble-translation text-caption text-medium-emphasis">
              {{ ex.translation }}
            </div>
            <div v-if="ex.score" class="bubble-score mt-1">
              <v-chip size="x-small" :color="(ex.score.overall as number) >= 80 ? 'success' : 'warning'" variant="tonal">
                {{ Math.round(ex.score.overall) }}/100
              </v-chip>
            </div>
          </div>
        </div>

        <!-- Current Turn Actions -->
        <div v-if="isPartnerTurn" class="text-center my-4">
          <v-btn
            variant="tonal"
            color="primary"
            :loading="partnerPlaying"
            prepend-icon="mdi-play-circle"
            @click="playPartnerLine">
            {{ $t('practice.roleplay.playPartner') }}
          </v-btn>
          <div class="text-caption text-medium-emphasis mt-2">
            {{ $t('practice.roleplay.yourTurn') }}
          </div>
        </div>

        <div v-if="isUserTurn" class="d-flex flex-column align-center my-4">
          <RecordButton
            size="large"
            @recorded="onRecorded"
            @error="(msg) => message.error(msg)" />
          <AudioVisualizer
            :audio-level="recorder.audioLevel"
            :is-active="recorder.status === 'recording'"
            class="mt-2 recording-visualizer"
            :height="36"
            :bars="20" />
        </div>

        <!-- Skip / Next -->
        <div class="d-flex justify-center mt-2">
          <v-btn
            v-if="!isPartnerTurn"
            variant="text"
            size="small"
            @click="nextExchange">
            {{ $t('practice.roleplay.next') }} →
          </v-btn>
        </div>
      </div>
    </template>
  </v-container>
</template>

<style scoped>
.roleplay-session {
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

.roleplay-workspace {
  max-width: 600px;
  margin: 0 auto;
}

.dialogue-area {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 400px;
  overflow-y: auto;
  padding: 4px;
}

.dialogue-bubble {
  padding: 10px 14px;
  border-radius: 12px;
  max-width: 80%;
}

.bubble-partner {
  align-self: flex-start;
  background: rgba(var(--v-theme-primary), 0.08);
  border-bottom-left-radius: 4px;
}

.bubble-user {
  align-self: flex-end;
  background: rgba(var(--v-theme-success), 0.08);
  border-bottom-right-radius: 4px;
}

.bubble-label {
  margin-bottom: 4px;
}

.bubble-text {
  line-height: 1.4;
}

.bubble-translation {
  margin-top: 4px;
}

.recording-visualizer {
  width: 100%;
  max-width: 360px;
}
</style>
