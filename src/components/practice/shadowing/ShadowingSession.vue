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
  defineProps<{ exerciseId?: string | null; text?: string | null }>(),
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
const audioRef = ref<HTMLAudioElement | null>(null);

type Step = 'listen' | 'record' | 'result';
const currentStep = ref<Step>('listen');

const currentIndex = computed(() => session.currentSentenceIndex);
const currentSentence = computed(() => sentences.value[currentIndex.value] || null);
const isComplete = computed(() => sentences.value.length > 0 && currentIndex.value >= sentences.value.length);
const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value < sentences.value.length - 1);

// Word popover
const popoverWord = ref('');
const popoverPhonemes = ref<string[]>([]);
const popoverVisible = ref(false);
const popoverX = ref(0);
const popoverY = ref(0);

async function onWordClick(word: string, event: MouseEvent) {
  try {
    const clean = word.replace(/[^a-zA-Z]/g, '');
    if (!clean) return;
    const phonemes = await invoke<string[] | null>('lookup_phonemes', { word: clean });
    if (phonemes && phonemes.length > 0) {
      popoverWord.value = clean;
      popoverPhonemes.value = phonemes;
      popoverX.value = event.clientX;
      popoverY.value = event.clientY;
      popoverVisible.value = true;
    }
  } catch { /* ignore */ }
}

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
      const list = await invoke<string[]>('split_sentences', { text: inputText });
      sentences.value = (list.length ? list : [inputText]).map((s, i) => ({
        id: `custom-${i}`, text: s, orderIndex: i,
      }));
    } else {
      const exercises = await invoke<Exercise[]>('list_exercises', {
        mode: 'shadowing', category: null, difficulty: null,
      });
      if (exercises.length > 0) {
        exercise.value = exercises[0];
        sentences.value = exercises[0].sentences;
      }
    }

    if (sentences.value.length > 0) {
      await session.startSession(exercise.value?.id || 'custom', 'shadowing');
      setTimeout(() => playReference(), 400);
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    loading.value = false;
  }
});

watch(currentIndex, () => {
  currentStep.value = 'listen';
  scoring.clearScore();
  recorder.reset();
  setTimeout(() => playReference(), 350);
});

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
      audioData: Array.from(audioData), durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });
    scoring.setScore(result);
    await invoke('record_attempt', {
      sessionId: session.currentSessionId, sentenceId: currentSentence.value.id,
      referenceText: currentSentence.value.text, audioData: Array.from(audioData),
      durationMs, whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });
    currentStep.value = 'result';
  } catch (e) {
    message.error(String(e));
  }
}

function startRecording() { currentStep.value = 'record'; }

function nextSentence() {
  if (hasNext.value) { session.nextSentence(); } else { completePractice(); }
}

function prevSentence() {
  if (hasPrev.value) { session.prevSentence(); }
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
      <v-progress-circular indeterminate size="28" color="primary" />
    </div>

    <template v-else-if="isComplete">
      <div class="text-center py-12">
        <div class="complete-check mb-4">✓</div>
        <h3 class="text-h4 font-weight-bold mb-2">Practice Complete</h3>
        <p class="text-body-1 text-medium-emphasis mb-6">Great session!</p>
        <v-btn color="primary" variant="tonal" size="large" class="px-8" @click="completePractice">
          Back to Hub
        </v-btn>
      </div>
    </template>

    <template v-else-if="currentSentence">
      <div class="session-content">
        <!-- Progress + Back -->
        <div class="d-flex align-center ga-3 mb-4">
          <v-btn variant="text" icon="mdi-arrow-left" size="small" color="secondary" @click="router.push('/practice')" />
          <div class="flex-grow-1">
            <SessionProgress :current="currentIndex + 1" :total="sentences.length" />
          </div>
        </div>

        <!-- Step pills -->
        <div class="d-flex justify-center ga-2 mb-4">
          <div :class="['step-pill', { 'step-active': currentStep === 'listen', 'step-done': false }]">
            <v-icon size="14" class="mr-1">1</v-icon> Listen
          </div>
          <div class="step-connector" />
          <div :class="['step-pill', { 'step-active': currentStep === 'record', 'step-done': currentStep === 'result' }]">
            <v-icon size="14" class="mr-1">{{ currentStep === 'result' ? 'mdi-check' : '2' }}</v-icon> Speak
          </div>
          <div class="step-connector" />
          <div :class="['step-pill', { 'step-active': currentStep === 'result' }]">
            <v-icon size="14" class="mr-1">3</v-icon> Review
          </div>
        </div>

        <!-- Sentence Card -->
        <v-card flat class="sentence-card mb-4">
          <v-card-text class="pa-5">
            <div class="sentence-text">
              <span v-for="(word, wi) in currentSentence.text.split(/(\s+)/)" :key="wi"
                :class="['word-span', { 'word-clickable': word.trim().length > 0 }]"
                @click="word.trim().length > 0 && onWordClick(word, $event)">{{ word }}</span>
            </div>
            <div v-if="currentSentence.translation" class="sentence-translation">
              {{ currentSentence.translation }}
            </div>
          </v-card-text>
        </v-card>

        <!-- Word Popover -->
        <WordPopover
          :word="popoverWord" :phonemes="popoverPhonemes"
          :visible="popoverVisible" :x="popoverX" :y="popoverY"
          @close="popoverVisible = false" />

        <!-- Listen Step -->
        <div v-if="currentStep === 'listen'" class="step-section">
          <div class="step-section__icon mb-3">🎧</div>
          <v-btn variant="tonal" color="primary" size="large" :loading="autoPlaying"
            prepend-icon="mdi-play-circle" class="px-6" @click="playReference">
            Listen
          </v-btn>
          <div class="text-caption text-medium-emphasis mt-3 mb-4">Listen to the reference, then record yourself</div>
          <v-btn variant="text" color="primary" @click="startRecording">
            Ready → Start Recording
          </v-btn>
        </div>

        <!-- Record Step -->
        <div v-if="currentStep === 'record'" class="step-section">
          <RecordButton size="large" @recorded="onRecorded" @error="(msg) => message.error(msg)" />
          <AudioVisualizer :audio-level="recorder.audioLevel" :is-active="recorder.status === 'recording'"
            class="mt-3 recording-viz" :height="36" :bars="20" />
          <v-btn variant="text" size="small" color="secondary" class="mt-2" @click="playReference">
            <v-icon size="14" class="mr-1">mdi-repeat</v-icon> Listen again
          </v-btn>
        </div>

        <!-- Result Step -->
        <div v-if="currentStep === 'result' && scoring.lastScore" class="step-section">
          <ScoreCard :score="scoring.lastScore" class="mb-3" />
          <WordScoreList :word-scores="scoring.lastScore.wordScores" class="mb-4" />
          <div class="d-flex justify-center ga-3">
            <v-btn variant="text" @click="prevSentence" :disabled="!hasPrev" class="px-4">← Prev</v-btn>
            <v-btn color="primary" variant="tonal" class="px-6" @click="nextSentence">
              {{ hasNext ? 'Next' : 'Finish' }} →
            </v-btn>
          </div>
        </div>
      </div>
    </template>
  </v-container>
</template>

<style scoped>
.shadowing-session {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
  background:
    radial-gradient(circle at 20% 10%, rgba(var(--v-theme-primary), 0.05), transparent 28%),
    radial-gradient(circle at 80% 90%, rgba(var(--v-theme-info), 0.04), transparent 24%),
    rgb(var(--v-theme-background));
}

.session-content {
  max-width: 560px;
  margin: 0 auto;
}

/* Step pills */
.step-pill {
  display: flex;
  align-items: center;
  padding: 4px 14px;
  border-radius: 20px;
  font-size: 0.78rem;
  font-weight: 500;
  background: rgba(var(--v-theme-glass), 0.4);
  color: rgba(var(--v-theme-on-surface), 0.4);
  border: 1px solid transparent;
  transition: all 0.2s ease;
}

.step-active {
  background: rgba(var(--v-theme-primary), 0.1);
  color: rgb(var(--v-theme-primary));
  border-color: rgba(var(--v-theme-primary), 0.2);
}

.step-done {
  background: rgba(var(--v-theme-success), 0.1);
  color: rgb(var(--v-theme-success));
  border-color: rgba(var(--v-theme-success), 0.2);
}

.step-connector {
  width: 16px;
  height: 1px;
  background: rgba(var(--v-theme-on-surface), 0.15);
}

/* Sentence card */
.sentence-card {
  border-radius: 20px !important;
  border: 1px solid rgba(var(--v-theme-glass-border), 0.35);
  background: rgba(var(--v-theme-glass), 0.72);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 2px 8px rgba(15, 23, 42, 0.04), 0 8px 24px rgba(15, 23, 42, 0.04);
}

.sentence-text {
  font-size: 1.15rem;
  font-weight: 500;
  line-height: 1.8;
  letter-spacing: -0.01em;
  color: rgb(var(--v-theme-on-surface));
}

.sentence-translation {
  margin-top: 8px;
  padding-top: 10px;
  border-top: 1px solid rgba(var(--v-theme-glass-border), 0.25);
  font-size: 0.85rem;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

/* Clickable words */
.word-clickable {
  cursor: pointer;
  border-bottom: 1px dashed rgba(var(--v-theme-primary), 0.2);
  transition: all 0.15s ease;
  padding: 0 1px;
  border-radius: 3px;
}
.word-clickable:hover {
  border-bottom-color: rgba(var(--v-theme-primary), 0.6);
  background: rgba(var(--v-theme-primary), 0.06);
}

/* Step sections */
.step-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
}

.step-section__icon {
  font-size: 2rem;
  line-height: 1;
}

.recording-viz {
  width: 100%;
  max-width: 320px;
}

/* Complete */
.complete-check {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: rgba(var(--v-theme-success), 0.12);
  color: rgb(var(--v-theme-success));
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 1.8rem;
  font-weight: 700;
}
</style>
