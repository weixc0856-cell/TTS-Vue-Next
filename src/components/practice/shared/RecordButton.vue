<script setup lang="ts">
import { computed } from 'vue';
import { useRecorderStore } from '../../../stores/practice/recorder';

const recorder = useRecorderStore();

const props = withDefaults(
  defineProps<{
    size?: 'small' | 'default' | 'large';
    disabled?: boolean;
  }>(),
  {
    size: 'default',
    disabled: false,
  },
);

const emit = defineEmits<{
  recorded: [audioData: Uint8Array, durationMs: number];
  error: [message: string];
}>();

const isRecording = computed(() => recorder.status === 'recording');
const isProcessing = computed(() => recorder.status === 'processing');

const icon = computed(() => {
  if (isRecording.value) return 'mdi-stop-circle';
  if (isProcessing.value) return 'mdi-loading';
  return 'mdi-record-circle';
});

const color = computed(() => {
  if (isRecording.value) return 'error';
  if (isProcessing.value) return 'warning';
  return 'primary';
});

const label = computed(() => {
  if (isRecording.value) return 'Click to stop recording';
  if (isProcessing.value) return 'Processing...';
  return 'Click to record';
});

async function handleClick() {
  if (props.disabled && !isRecording.value) return;

  if (!isRecording.value) {
    // Start recording
    await recorder.startRecording();
    startLevelPolling();
  } else {
    // Stop recording
    stopLevelPolling();
    const audioData = await recorder.stopRecording();
    if (audioData) {
      emit('recorded', audioData, recorder.durationMs);
    }
    if (recorder.error) {
      emit('error', recorder.error);
    }
  }
}

let levelInterval: ReturnType<typeof setInterval> | null = null;

function startLevelPolling() {
  levelInterval = setInterval(() => {
    recorder.pollAudioLevel();
  }, 100);
}

function stopLevelPolling() {
  if (levelInterval) {
    clearInterval(levelInterval);
    levelInterval = null;
  }
}
</script>

<template>
  <div class="record-button-wrapper">
    <v-btn
      :icon="isRecording ? 'mdi-stop-circle' : 'mdi-record-circle'"
      :color="color"
      :size="size"
      :loading="isProcessing"
      :disabled="disabled && !isRecording"
      :class="{ 'recording-active': isRecording }"
      class="record-btn"
      @click="handleClick">
      <v-icon :size="size === 'large' ? 48 : size === 'small' ? 24 : 36">
        {{ icon }}
      </v-icon>
    </v-btn>
    <div class="record-label text-caption text-medium-emphasis">
      {{ label }}
    </div>
    <div v-if="isRecording" class="audio-level-bar">
      <div
        class="audio-level-fill"
        :style="{ width: `${recorder.audioLevel * 100}%` }" />
    </div>
  </div>
</template>

<style scoped>
.record-button-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.record-btn {
  transition: all 0.2s ease;
}

.record-btn.recording-active {
  animation: pulse 1.2s ease-in-out infinite;
  box-shadow: 0 0 0 4px rgba(var(--v-theme-error), 0.3);
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

.audio-level-bar {
  width: 80px;
  height: 4px;
  background: rgba(var(--v-theme-on-surface), 0.1);
  border-radius: 4px;
  overflow: hidden;
}

.audio-level-fill {
  height: 100%;
  background: rgb(var(--v-theme-error));
  border-radius: 4px;
  transition: width 0.1s ease;
}

.record-label {
  user-select: none;
}
</style>
