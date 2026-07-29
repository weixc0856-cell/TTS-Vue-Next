<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useMessage } from 'vuetify-message-vue3';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../../stores/settings';
import { useSessionStore } from '../../../stores/practice/session';
import { useRecorderStore } from '../../../stores/practice/recorder';
import { useScoringStore } from '../../../stores/practice/scoring';
import RecordButton from '../shared/RecordButton.vue';
import AudioVisualizer from '../shared/AudioVisualizer.vue';
import ScoreCard from '../shared/ScoreCard.vue';
import WordScoreList from '../shared/WordScoreList.vue';
import SessionProgress from '../shared/SessionProgress.vue';
import type { Exercise, Sentence, ScoreResult } from '../../../types/practice';

const props = withDefaults(
  defineProps<{
    exerciseId?: string | null;
    text?: string | null;
  }>(),
  { exerciseId: null, text: null },
);

const route = useRoute();
const router = useRouter();
const message = useMessage();
const settingsStore = useSettingsStore();
const session = useSessionStore();
const recorder = useRecorderStore();
const scoring = useScoringStore();

const exercise = ref<Exercise | null>(null);
const sentences = ref<Sentence[]>([]);
const loading = ref(true);

const currentIndex = computed(() => session.currentSentenceIndex);
const currentSentence = computed(() => sentences.value[currentIndex.value] || null);
const isComplete = computed(() => currentIndex.value >= sentences.value.length);
const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value < sentences.value.length);

onMounted(async () => {
  try {
    // Load from route query (passed via navigation)
    const id = props.exerciseId || route.query.exerciseId as string;
    const inputText = props.text || route.query.text as string;

    if (id) {
      const ex = await invoke<Exercise>('get_exercise_detail', { id });
      exercise.value = ex;
      sentences.value = ex.sentences;
    } else if (inputText) {
      // Create custom exercise from text
      const sentences_list = await invoke<string[]>('split_sentences', { text: inputText });
      if (sentences_list.length === 0) {
        sentences_list.push(inputText);
      }
      // No translations for user text
      sentences.value = sentences_list.map((s, i) => ({
        id: `custom-${i}`,
        text: s,
        orderIndex: i,
      }));
    } else {
      // No exercise specified, load all shadowing exercises
      const exercises = await invoke<Exercise[]>('list_exercises', {
        mode: 'shadowing',
        category: null,
        difficulty: null,
      });
      if (exercises.length > 0) {
        const ex = exercises[0];
        exercise.value = ex;
        sentences.value = ex.sentences;
      }
    }

    if (sentences.value.length > 0) {
      await session.startSession(
        exercise.value?.id || 'custom',
        'shadowing',
      );
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    loading.value = false;
  }
});

const audioRef = ref<HTMLAudioElement | null>(null);

async function playReference() {
  if (!currentSentence.value) return;
  try {
    const audioData = await invoke<number[]>('tts_convert', {
      params: {
        text: currentSentence.value.text,
        voice: 'en-US-EmmaMultilingualNeural',
        rate: '+0%',
        pitch: '+0Hz',
        volume: '+0%',
        format: 'audio-24khz-48kbitrate-mono-mp3',
        task_id: `shadowing-${Date.now()}`,
        max_retries: 2,
      },
    });

    // Play the returned audio
    const audioBytes = new Uint8Array(audioData);
    const blob = new Blob([audioBytes], { type: 'audio/mpeg' });
    const url = URL.createObjectURL(blob);

    if (audioRef.value) {
      audioRef.value.src = url;
      audioRef.value.load();
      await audioRef.value.play().catch(() => {
        // Browser may block autoplay, that's ok
      });
    }
  } catch (e) {
    message.error(String(e));
  }
}

async function onRecorded(audioData: Uint8Array, durationMs: number) {
  if (!currentSentence.value || !session.currentSessionId) return;

  try {
    // Transcribe and score
    const result = await invoke<ScoreResult>('transcribe_and_score', {
      referenceText: currentSentence.value.text,
      audioData: Array.from(audioData),
      durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });

    scoring.setScore(result);

    // Record the attempt
    await invoke('record_attempt', {
      sessionId: session.currentSessionId,
      sentenceId: currentSentence.value.id,
      referenceText: currentSentence.value.text,
      audioData: Array.from(audioData),
      durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });
  } catch (e) {
    message.error(String(e));
  }
}

function nextSentence() {
  if (hasNext.value) {
    session.nextSentence();
    scoring.clearScore();
    recorder.reset();
  }
}

function prevSentence() {
  if (hasPrev.value) {
    session.prevSentence();
    scoring.clearScore();
    recorder.reset();
  }
}

async function completePractice() {
  try {
    const score = await session.endSession();
    message.success(`Practice complete! Score: ${Math.round(score)}/100`);
  } catch {
    // Session may not exist
  }
  router.push('/practice');
}
</script>

<template>
  <v-container fluid class="shadowing-session">
    <div v-if="loading" class="d-flex justify-center align-center py-12">
      <v-progress-circular indeterminate size="32" />
    </div>

    <template v-else-if="isComplete">
      <div class="text-center py-8">
        <v-icon size="56" color="success" class="mb-3">mdi-check-circle</v-icon>
        <h3 class="text-h5 mb-2">{{ $t('practice.shadowing.complete') }}</h3>
        <v-btn color="primary" variant="tonal" @click="completePractice">
          {{ $t('practice.actions.backToHub') }}
        </v-btn>
      </div>
    </template>

    <template v-else-if="currentSentence">
      <div class="shadowing-workspace">
        <!-- Progress -->
        <SessionProgress :current="currentIndex + 1" :total="sentences.length" />

        <!-- Sentence Display -->
        <v-card flat class="sentence-panel glass-panel my-4">
          <v-card-text class="text-center py-4">
            <div class="text-h6 font-weight-medium mb-2">
              {{ currentSentence.text }}
            </div>
            <div v-if="currentSentence.translation" class="text-body-2 text-medium-emphasis">
              {{ currentSentence.translation }}
            </div>
          </v-card-text>
        </v-card>

        <!-- Play Reference -->
        <div class="d-flex justify-center mb-4">
          <audio ref="audioRef" preload="auto" style="display:none" />
          <v-btn
            variant="tonal"
            color="primary"
            prepend-icon="mdi-play-circle"
            @click="playReference">
            {{ $t('practice.shadowing.playReference') }}
          </v-btn>
        </div>

        <!-- Recording -->
        <div class="d-flex flex-column align-center mb-4">
          <RecordButton
            size="large"
            @recorded="onRecorded"
            @error="(msg) => message.error(msg)" />
          <AudioVisualizer
            :audio-level="recorder.audioLevel"
            :is-active="recorder.status === 'recording'"
            class="mt-2 recording-visualizer"
            :height="40"
            :bars="24" />
        </div>

        <!-- Score Result -->
        <ScoreCard
          v-if="scoring.lastScore"
          :score="scoring.lastScore"
          class="mb-4" />

        <WordScoreList
          v-if="scoring.lastScore"
          :word-scores="scoring.lastScore.wordScores"
          class="mb-4" />

        <!-- Navigation -->
        <div class="d-flex justify-space-between">
          <v-btn
            variant="text"
            :disabled="!hasPrev"
            @click="prevSentence">
            ← {{ $t('practice.shadowing.sentence') }}
          </v-btn>
          <v-btn
            color="primary"
            variant="tonal"
            @click="nextSentence">
            {{ hasNext ? $t('practice.shadowing.next') : $t('practice.shadowing.complete') }} →
          </v-btn>
        </div>
      </div>
    </template>

    <div v-else class="text-center py-8 text-medium-emphasis">
      No sentences available
    </div>
  </v-container>
</template>

<style scoped>
.shadowing-session {
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

.shadowing-workspace {
  max-width: 650px;
  margin: 0 auto;
}

.sentence-panel {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background:
    radial-gradient(
      circle at top right,
      rgba(var(--v-theme-primary), 0.1),
      transparent 30%
    ),
    rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}

.recording-visualizer {
  width: 100%;
  max-width: 400px;
}
</style>
