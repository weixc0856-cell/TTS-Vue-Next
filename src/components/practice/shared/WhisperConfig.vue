<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from '../../../stores/settings';

const settingsStore = useSettingsStore();
const autoDetected = ref(false);

const modelSizeOptions = [
  { title: 'Base.en (142MB, recommended)', value: 'base.en' },
  { title: 'Small.en (466MB, more accurate)', value: 'small.en' },
  { title: 'Tiny.en (75MB, faster)', value: 'tiny.en' },
];

const isConfigured = computed(() => {
  return !!(settingsStore.whisperBinPath && settingsStore.whisperModelPath);
});

const statusText = computed(() => {
  if (isConfigured.value) return '✅ Whisper configured';
  if (autoDetected.value) return '⚠️ Model file detected but binary not set';
  return '⚠️ Not configured — scoring will use placeholder';
});

const statusColor = computed(() => {
  return isConfigured.value ? 'success' : autoDetected.value ? 'info' : 'warning';
});

const downloadLink = computed(() => {
  const size = settingsStore.whisperModelSize || 'base.en';
  return `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-${size}.bin`;
});

// ── Auto-detect ──

async function detectPaths() {
  // Try to find a ggml-*.bin model in common locations
  const candidates = [
    'D:\\Project\\AuraVoice\\ggml-base.en.bin',
    'D:\\Project\\AuraVoice\\ggml-small.en.bin',
    'D:\\Project\\AuraVoice\\ggml-tiny.en.bin',
    'C:\\tools\\whisper.cpp\\ggml-base.en.bin',
    'C:\\tools\\whisper.cpp\\ggml-small.en.bin',
    './ggml-base.en.bin',
    './ggml-small.en.bin',
  ];

  const fs = await import('@tauri-apps/plugin-fs');

  // Check if model already set but empty
  if (!settingsStore.whisperModelPath) {
    for (const candidate of candidates) {
      try {
        const exists = await fs.exists(candidate);
        if (exists) {
          settingsStore.updateWhisperModelPath(candidate);
          autoDetected.value = true;
          break;
        }
      } catch {
        // Try next
      }
    }
  }

  // Try to find whisper-cli binary in common locations
  if (!settingsStore.whisperBinPath) {
    const binCandidates = [
      'C:\\tools\\whisper.cpp\\whisper-cli.exe',
      'C:\\whisper.cpp\\whisper-cli.exe',
      '.\\whisper-cli.exe',
    ];
    for (const candidate of binCandidates) {
      try {
        const exists = await fs.exists(candidate);
        if (exists) {
          settingsStore.updateWhisperBinPath(candidate);
          break;
        }
      } catch {
        // Try next
      }
    }
  }
}

onMounted(() => {
  detectPaths();
});

async function selectWhisperBinary() {
  try {
    const path = await open({
      title: 'Select whisper-cli executable',
      filters: [
        { name: 'whisper-cli', extensions: ['exe', ''] },
      ],
      multiple: false,
    });
    if (path) {
      settingsStore.updateWhisperBinPath(path);
    }
  } catch (e) {
    console.error('Failed to select whisper binary:', e);
  }
}

async function selectWhisperModel() {
  try {
    const path = await open({
      title: 'Select Whisper model file (ggml-*.bin)',
      filters: [
        { name: 'Whisper model', extensions: ['bin'] },
      ],
      multiple: false,
    });
    if (path) {
      settingsStore.updateWhisperModelPath(path);
    }
  } catch (e) {
    console.error('Failed to select model:', e);
  }
}

function clearPaths() {
  settingsStore.updateWhisperBinPath('');
  settingsStore.updateWhisperModelPath('');
  autoDetected.value = false;
}
</script>

<template>
  <section class="settings-group settings-group--full">
    <div class="settings-group__header">
      <v-icon size="18" color="primary">mdi-microphone</v-icon>
      <span class="text-subtitle-1 font-weight-medium">
        Whisper Speech Recognition
      </span>
      <v-chip
        :color="statusColor"
        size="x-small"
        variant="tonal"
        class="ml-2">
        {{ isConfigured ? 'Configured' : autoDetected ? 'Partial' : 'Not configured' }}
      </v-chip>
    </div>

    <div class="text-caption text-medium-emphasis mb-3">
      {{ statusText }}
    </div>

    <!-- Binary Path -->
    <div class="settings-field settings-field--full">
      <v-text-field
        :model-value="settingsStore.whisperBinPath"
        label="whisper-cli binary path"
        placeholder="C:/tools/whisper.cpp/whisper-cli.exe"
        density="comfortable"
        readonly
        append-inner-icon="mdi-file-search"
        @click="selectWhisperBinary" />
      <div class="text-caption text-medium-emphasis mt-1">
        Download from
        <a
          href="https://github.com/ggerganov/whisper.cpp/releases"
          target="_blank"
          class="text-primary">
          github.com/ggerganov/whisper.cpp
        </a>
      </div>
    </div>

    <!-- Model Path -->
    <div class="settings-field settings-field--full mt-2">
      <v-text-field
        :model-value="settingsStore.whisperModelPath"
        label="Whisper model file"
        :placeholder="`ggml-${settingsStore.whisperModelSize}.bin`"
        density="comfortable"
        readonly
        append-inner-icon="mdi-file-search"
        @click="selectWhisperModel" />
    </div>

    <!-- Model Size Selector -->
    <div class="settings-field mt-2" style="max-width: 280px">
      <v-select
        :model-value="settingsStore.whisperModelSize"
        :items="modelSizeOptions"
        density="comfortable"
        label="Model size"
        @update:model-value="
          (v) => settingsStore.updateWhisperModelSize(v)
        " />
    </div>

    <!-- Download helper -->
    <div class="mt-2">
      <v-btn
        variant="tonal"
        size="small"
        color="primary"
        prepend-icon="mdi-download"
        :href="downloadLink"
        target="_blank">
        Download {{ settingsStore.whisperModelSize }} model ({{ downloadLink.split('/').pop() }})
      </v-btn>
      <span class="text-caption text-medium-emphasis ml-2">
        ~{{ settingsStore.whisperModelSize === 'small.en' ? '466' : settingsStore.whisperModelSize === 'base.en' ? '142' : '75' }}MB
      </span>
    </div>

    <!-- Status text after detection -->
    <div
      v-if="autoDetected && !isConfigured"
      class="text-caption text-success mt-1">
      ✅ Found ggml-base.en.bin! Still need to set whisper-cli binary path.
    </div>

    <!-- Clear button -->
    <div class="mt-2">
      <v-btn
        v-if="isConfigured"
        variant="text"
        size="small"
        color="error"
        @click="clearPaths">
        Clear paths
      </v-btn>
    </div>
  </section>
</template>

<style scoped>
.settings-group--full {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 20px;
  background: rgba(var(--v-theme-surface), 0.6);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}

.settings-group__header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.settings-field {
  margin-bottom: 4px;
}

.settings-field--full {
  width: 100%;
}
</style>
