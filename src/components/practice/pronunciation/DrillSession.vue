<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useMessage } from 'vuetify-message-vue3';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../../stores/settings';
import { useRecorderStore } from '../../../stores/practice/recorder';
import RecordButton from '../shared/RecordButton.vue';
import AudioVisualizer from '../shared/AudioVisualizer.vue';
import ScoreCard from '../shared/ScoreCard.vue';
import type { ScoreResult } from '../../../types/practice';

const router = useRouter();
const message = useMessage();
const settingsStore = useSettingsStore();
const recorder = useRecorderStore();

interface DrillWord {
  word: string;
  phonemes: string[];
  difficulty: 'easy' | 'medium' | 'hard';
  hint: string;
}

const drillWords = ref<DrillWord[]>([]);
const currentIndex = ref(0);
const loading = ref(true);
const lastScore = ref<ScoreResult | null>(null);
const audioRef = ref<HTMLAudioElement | null>(null);

const currentWord = computed(() => drillWords.value[currentIndex.value] || null);
const progress = computed(() => ({
  current: currentIndex.value + 1,
  total: drillWords.value.length,
}));
const isComplete = computed(() => currentIndex.value >= drillWords.value.length);

const drillLibrary: DrillWord[] = [
  // TH sounds
  { word: 'think', phonemes: ['TH', 'IH1', 'NG', 'K'], difficulty: 'hard', hint: 'Put your tongue between your teeth' },
  { word: 'that', phonemes: ['DH', 'AE1', 'T'], difficulty: 'hard', hint: 'Voiced TH — vibrate your throat' },
  { word: 'three', phonemes: ['TH', 'R', 'IY1'], difficulty: 'hard', hint: 'Unvoiced TH + R sound' },
  { word: 'the', phonemes: ['DH', 'AH0'], difficulty: 'easy', hint: 'Quick voiced TH' },
  { word: 'through', phonemes: ['TH', 'R', 'UW1'], difficulty: 'hard', hint: 'TH + R + long U' },

  // R vs L
  { word: 'right', phonemes: ['R', 'AY1', 'T'], difficulty: 'medium', hint: 'Curl your tongue back for R' },
  { word: 'light', phonemes: ['L', 'AY1', 'T'], difficulty: 'medium', hint: 'Tongue touches the roof for L' },
  { word: 'rice', phonemes: ['R', 'AY1', 'S'], difficulty: 'medium', hint: 'R vs L — compare with "lice"' },
  { word: 'lice', phonemes: ['L', 'AY1', 'S'], difficulty: 'medium', hint: 'L sound — tongue to palate' },
  { word: 'collect', phonemes: ['K', 'AH0', 'L', 'EH1', 'K', 'T'], difficulty: 'medium', hint: 'Clear L in the middle' },

  // Minimal pairs
  { word: 'ship', phonemes: ['SH', 'IH1', 'P'], difficulty: 'easy', hint: 'Short i sound' },
  { word: 'sheep', phonemes: ['SH', 'IY1', 'P'], difficulty: 'easy', hint: 'Long ee sound — stretch the vowel' },
  { word: 'bit', phonemes: ['B', 'IH1', 'T'], difficulty: 'easy', hint: 'Short i' },
  { word: 'beat', phonemes: ['B', 'IY1', 'T'], difficulty: 'easy', hint: 'Long ee' },
  { word: 'full', phonemes: ['F', 'UH1', 'L'], difficulty: 'medium', hint: 'Short u sound' },
  { word: 'fool', phonemes: ['F', 'UW1', 'L'], difficulty: 'medium', hint: 'Long oo — round your lips' },

  // Silent letters
  { word: 'knife', phonemes: ['N', 'AY1', 'F'], difficulty: 'hard', hint: 'The K is silent!' },
  { word: 'write', phonemes: ['R', 'AY1', 'T'], difficulty: 'hard', hint: 'The W is silent!' },
  { word: 'hour', phonemes: ['AW1', 'ER0'], difficulty: 'hard', hint: 'The H is silent!' },
  { word: 'listen', phonemes: ['L', 'IH1', 'S', 'AH0', 'N'], difficulty: 'medium', hint: 'The T is silent!' },

  // Vowels
  { word: 'cat', phonemes: ['K', 'AE1', 'T'], difficulty: 'easy', hint: 'Open your mouth wide' },
  { word: 'cut', phonemes: ['K', 'AH1', 'T'], difficulty: 'easy', hint: 'Shorter, more relaxed' },
  { word: 'cot', phonemes: ['K', 'AA1', 'T'], difficulty: 'easy', hint: 'Round your mouth' },
  { word: 'kite', phonemes: ['K', 'AY1', 'T'], difficulty: 'easy', hint: 'Glide from AH to EE' },
];

onMounted(() => {
  // Shuffle and pick 10 words per session
  const shuffled = [...drillLibrary].sort(() => Math.random() - 0.5);
  drillWords.value = shuffled.slice(0, 10);
  loading.value = false;
});

async function playReference() {
  if (!currentWord.value) return;
  try {
    const audioData = await invoke<number[]>('tts_convert', {
      params: {
        text: currentWord.value.word,
        voice: 'en-US-EmmaMultilingualNeural',
        rate: '-20%',
        pitch: '+0Hz', volume: '+0%',
        format: 'audio-24khz-48kbitrate-mono-mp3',
        task_id: `drill-${Date.now()}`,
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
  }
}

async function onRecorded(audioData: Uint8Array, durationMs: number) {
  if (!currentWord.value) return;

  try {
    const result = await invoke<ScoreResult>('transcribe_and_score', {
      referenceText: currentWord.value.word,
      audioData: Array.from(audioData),
      durationMs,
      whisperBinPath: settingsStore.whisperBinPath || null,
      whisperModelPath: settingsStore.whisperModelPath || null,
    });

    lastScore.value = result;
  } catch (e) {
    message.error(String(e));
  }
}

function nextWord() {
  if (currentIndex.value < drillWords.value.length - 1) {
    currentIndex.value++;
    lastScore.value = null;
    recorder.reset();
  }
}

function completeDrill() {
  router.push('/practice');
}
</script>

<template>
  <v-container fluid class="drill-session">
    <audio ref="audioRef" preload="auto" style="display:none" />

    <div v-if="loading" class="d-flex justify-center align-center py-12">
      <v-progress-circular indeterminate size="32" />
    </div>

    <template v-else-if="isComplete">
      <div class="text-center py-8">
        <v-icon size="56" color="success" class="mb-3">mdi-check-circle</v-icon>
        <h3 class="text-h5 mb-2">Drill Complete!</h3>
        <p class="text-body-2 text-medium-emphasis mb-4">Great work practicing those sounds.</p>
        <v-btn color="primary" variant="tonal" @click="completeDrill">
          Back to Hub
        </v-btn>
      </div>
    </template>

    <template v-else-if="currentWord">
      <div class="drill-workspace">
        <!-- Progress -->
        <div class="d-flex align-center ga-2 mb-4">
          <v-progress-linear
            :model-value="(progress.current / progress.total) * 100"
            color="primary"
            height="4"
            rounded />
          <span class="text-caption text-medium-emphasis">{{ progress.current }}/{{ progress.total }}</span>
        </div>

        <!-- Word display -->
        <v-card flat class="word-panel glass-panel my-4 text-center py-4">
          <div class="text-h4 font-weight-bold mb-2">{{ currentWord.word }}</div>
          <div class="text-body-2 text-medium-emphasis font-family-mono">
            /{{ currentWord.phonemes.join('') }}/
          </div>
          <div class="text-caption mt-2">
            <v-chip
              size="x-small"
              :color="currentWord.difficulty === 'hard' ? 'error' : currentWord.difficulty === 'medium' ? 'warning' : 'success'"
              variant="tonal">
              {{ currentWord.difficulty }}
            </v-chip>
          </div>
        </v-card>

        <!-- Phoneme breakdown -->
        <div class="phoneme-breakdown d-flex justify-center ga-1 mb-4 flex-wrap">
          <v-chip
            v-for="(p, i) in currentWord.phonemes"
            :key="i"
            size="small"
            variant="outlined"
            color="primary"
            class="phoneme-chip">
            {{ p }}
          </v-chip>
        </div>

        <!-- Hint -->
        <div class="text-center text-body-2 text-medium-emphasis mb-4">
          💡 {{ currentWord.hint }}
        </div>

        <!-- Audio + Record -->
        <div class="d-flex justify-center ga-3 mb-4">
          <v-btn variant="tonal" color="primary" prepend-icon="mdi-play-circle" @click="playReference">
            Listen
          </v-btn>
          <RecordButton size="default" @recorded="onRecorded" @error="(msg) => message.error(msg)" />
        </div>

        <AudioVisualizer
          :audio-level="recorder.audioLevel"
          :is-active="recorder.status === 'recording'"
          class="mb-4 recording-visualizer"
          :height="36"
          :bars="20" />

        <!-- Score -->
        <ScoreCard v-if="lastScore" :score="lastScore" class="mb-3" />

        <!-- Navigation -->
        <div class="d-flex justify-center mt-4">
          <v-btn color="primary" variant="tonal" @click="nextWord">
            {{ currentIndex < drillWords.length - 1 ? 'Next Word →' : 'Finish →' }}
          </v-btn>
        </div>
      </div>
    </template>
  </v-container>
</template>

<style scoped>
.drill-session {
  padding: 10px;
  height: 100%;
  background:
    radial-gradient(circle at top right, rgba(var(--v-theme-primary), 0.08), transparent 22%),
    linear-gradient(180deg, rgba(var(--v-theme-surface), 1), rgba(var(--v-theme-surface), 0.98));
}

.drill-workspace {
  max-width: 550px;
  margin: 0 auto;
}

.word-panel {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  background:
    radial-gradient(circle at top right, rgba(var(--v-theme-primary), 0.1), transparent 30%),
    rgba(var(--v-theme-surface), 0.78);
  backdrop-filter: blur(18px);
}

.phoneme-breakdown {
  gap: 4px;
}

.phoneme-chip {
  font-family: 'Courier New', monospace;
  letter-spacing: 1px;
}

.recording-visualizer {
  width: 100%;
  max-width: 350px;
  margin: 0 auto;
}
</style>
