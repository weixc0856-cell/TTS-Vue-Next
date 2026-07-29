<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
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
import WordPopover from '../shared/WordPopover.vue';
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
const autoPlaying = ref(false);

type Step = 'listen' | 'record' | 'result';
const currentStep = ref<Step>('listen');

const currentIndex = computed(() => session.currentSentenceIndex);
const currentSentence = computed(() => sentences.value[currentIndex.value] || null);
const isComplete = computed(() => sentences.value.length > 0 && currentIndex.value >= sentences.value.length);
const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value < sentences.value.length - 1);

onMounted(async () => {
  try {
    try { await invoke('seed_content'); } catch { /* ok */ }

    const id = props.exerciseId || route.query.exerciseId as string;
    const inputText = props.text || route.query.text as string;

    if (id) {
      const ex = await invoke<Exercise>('get_exercise_detail', { id });
      exercise.value = ex;
      sentences.value = ex.sentences;
    } else if (inputText) {
      const sentenceList = await invoke<string[]>('split_sentences', { text: inputText });
      sentences.value = (sentenceList.length ? sentenceList : [inputText]).map((s, i) => ({
        id: `custom-${i}`, text: s, orderIndex: i,
      }));
    } else {
      const exercises = await invoke<Exercise[]>('list_exercises', { mode: 'shadowing', category: null, difficulty: null });
      if (exercises.length > 0) {
        const ex = exercises[0];
        exercise.value = ex;
        sentences.value = ex.sentences;
      }
    }

    if (sentences.value.length > 0) {
      await session.startSession(exercise.value?.id || 'custom', 'shadowing');
      // Auto-play first sentence
      setTimeout(() => playReference(), 300);
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    loading.value = false;
  }
});

// Watch for exercise selection changes
watch(currentIndex, () => {
  currentStep.value = 'listen';
  setTimeout(() => playReference(), 200);
});

// Word popover
const popoverWord = ref('');
const popoverPhonemes = ref<string[]>([]);
const popoverVisible = ref(false);
const popoverX = ref(0);
const popoverY = ref(0);

async function onWordClick(word: string, event: MouseEvent) {
  try {
    const phonemes = await invoke<string[] | null>('lookup_phonemes', { word: word.replace(/[^a-zA-Z]/g, '') });
    if (phonemes && phonemes.length > 0) {
      popoverWord.value = word.replace(/[^a-zA-Z]/g, '');
      popoverPhonemes.value = phonemes;
      popoverX.value = event.clientX;
      popoverY.value = event.clientY;
      popoverVisible.value = true;
    }
  } catch { /* silently ignore */ }
}

const audioRef = ref<HTMLAudioElement | null>(null);

async function playReference() {
  if (!currentSentence.value) return;
  autoPlaying.value = true;
  try {
    const audioData = await invoke<number[]>('tts_convert', {
      params: {
        text: currentSentence.value.text,
        voice: 'en-US-EmmaMultilingualNeural',
        rate: '+0%', pitch: '+0Hz', volume: '+0%',
        format: 'audio-24khz-48kbitrate-mono-mp3',
        task_id: `shadowing-${Date.now()}`,
        max_retries: 2,
      },
    });

    const bytes = new Uint8Array(audioData);
    let binary = '';
    for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
    const dataUri = `data:audio/mpeg;base64,${btoa(binary)}`;

    if (audioRef.value) {
      audioRef.value.src = dataUri;
      await audioRef.value.play().catch(() => {});
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    autoPlaying.value = false;
  }
}

async function onRecorded(audioData: Uint8Array, durationMs: number) {
  if (!currentSentence.value || !session.currentSessionId) return;

  try {
    const result = await invoke<ScoreResult>('transcribe_and_score', {
      referenceText: currentSentence.value.text,
      audioData: Array.from(audioData),
      durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });

    scoring.setScore(result);

    await invoke('record_attempt', {
      sessionId: session.currentSessionId,
      sentenceId: currentSentence.value.id,
      referenceText: currentSentence.value.text,
      audioData: Array.from(audioData),
      durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });

    currentStep.value = 'result';
  } catch (e) {
    message.error(String(e));
  }
}

function startRecording() {
  currentStep.value = 'record';
}

function nextSentence() {
  if (hasNext.value) {
    session.nextSentence();
    scoring.clearScore();
    recorder.reset();
  } else {
    completePractice();
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
  } catch { /* ok */ }
  router.push('/practice');
}
</script>

<template>
  <v-container fluid class="shadowing-session">
    <audio ref="audioRef" preload="auto" style="display:none" />

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
        <SessionProgress :current="currentIndex + 1" :total="sentences.length" />

        <!-- Step indicator -->
        <div class="step-indicator d-flex justify-center ga-2 my-2">
          <v-chip size="x-small" :color="currentStep === 'listen' ? 'primary' : 'success'" variant="tonal">
            1. Listen
          </v-chip>
          <v-chip size="x-small" :color="currentStep === 'record' ? 'error' : currentStep === 'result' ? 'success' : ''" variant="tonal">
            2. Record
          </v-chip>
          <v-chip size="x-small" :color="currentStep === 'result' ? 'success' : ''" variant="tonal">
            3. Score
          </v-chip>
        </div>

        <!-- Sentence Display with clickable words -->
        <v-card flat class="sentence-panel glass-panel my-3">
          <v-card-text class="text-center py-4">
            <div class="word-container text-h6 font-weight-medium mb-2">
              <span
                v-for="(word, wi) in currentSentence.text.split(/(\s+)/)"
                :key="wi"
                :class="['word-span', { 'word-clickable': word.trim().length > 0 }]"
                @click="word.trim().length > 0 && onWordClick(word, $event)">
                {{ word }}
              </span>
            </div>
            <div v-if="currentSentence.translation" class="text-body-2 text-medium-emphasis">
              {{ currentSentence.translation }}
            </div>
          </v-card-text>
        </v-card>

        <!-- Word Phoneme Popover -->
        <WordPopover
          :word="popoverWord"
          :phonemes="popoverPhonemes"
          :visible="popoverVisible"
          :x="popoverX"
          :y="popoverY"
          @close="popoverVisible = false" />

        <!-- Listen Step -->
        <div v-if="currentStep === 'listen'" class="text-center my-4">
          <v-btn
            variant="tonal"
            color="primary"
            size="large"
            :loading="autoPlaying"
            prepend-icon="mdi-play-circle"
            @click="playReference">
            {{ $t('practice.shadowing.playReference') }}
          </v-btn>
          <div class="text-caption text-medium-emphasis mt-2">
            Listen to the reference audio, then click Record.
          </div>
          <v-btn variant="text" color="primary" class="mt-2" @click="startRecording">
            Ready — Start Recording →
          </v-btn>
        </div>

        <!-- Record Step -->
        <div v-if="currentStep === 'record'" class="d-flex flex-column align-center my-4">
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
          <v-btn variant="text" size="small" class="mt-2" @click="playReference">
            🔄 Listen again
          </v-btn>
        </div>

        <!-- Result Step -->
        <template v-if="currentStep === 'result' && scoring.lastScore">
          <ScoreCard :score="scoring.lastScore" class="mb-3" />
          <WordScoreList :word-scores="scoring.lastScore.wordScores" class="mb-3" />

          <div class="d-flex justify-center ga-3">
            <v-btn variant="text" @click="prevSentence" :disabled="!hasPrev">
              ← Prev
            </v-btn>
            <v-btn
              color="primary"
              variant="tonal"
              @click="nextSentence">
              {{ hasNext ? 'Next →' : 'Finish →' }}
            </v-btn>
          </div>
        </template>
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
    radial-gradient(circle at top right, rgba(var(--v-theme-primary), 0.08), transparent 22%),
    linear-gradient(180deg, rgba(var(--v-theme-surface), 1), rgba(var(--v-theme-surface), 0.98));
}

.shadowing-workspace {
  max-width: 650px;
  margin: 0 auto;
}

.sentence-panel {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background:
    radial-gradient(circle at top right, rgba(var(--v-theme-primary), 0.1), transparent 30%),
    rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}

.recording-visualizer {
  width: 100%;
  max-width: 400px;
}

.word-container {
  line-height: 1.8;
}

.word-clickable {
  cursor: pointer;
  border-bottom: 1px dashed rgba(var(--v-theme-primary), 0.25);
  transition: border-color 0.15s ease, background-color 0.15s ease;
  padding: 0 1px;
  border-radius: 3px;
}

.word-clickable:hover {
  border-bottom-color: rgba(var(--v-theme-primary), 0.7);
  background: rgba(var(--v-theme-primary), 0.05);
}
</style>
