<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  word: string;
  phonemes: string[];
  visible: boolean;
  x: number;
  y: number;
}>();

const emit = defineEmits<{
  close: [];
}>();

const phonemeDisplay = computed(() => {
  return '/' + props.phonemes.join('') + '/';
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="word-popover-overlay"
      @click="emit('close')"
      @touchstart="emit('close')" />
    <div
      v-if="visible"
      class="word-popover"
      :style="{ left: `${x}px`, top: `${y}px` }">
      <div class="word-popover__word font-weight-bold">{{ word }}</div>
      <div class="word-popover__phonemes font-family-mono text-body-2 mt-1">
        {{ phonemeDisplay }}
      </div>
      <div class="word-popover__chips mt-2">
        <v-chip
          v-for="(p, i) in phonemes"
          :key="i"
          size="x-small"
          variant="outlined"
          color="primary"
          class="mr-1">
          {{ p }}
        </v-chip>
      </div>
      <v-btn
        size="x-small"
        variant="text"
        icon="mdi-close"
        class="word-popover__close"
        @click="emit('close')" />
    </div>
  </Teleport>
</template>

<style scoped>
.word-popover-overlay {
  position: fixed;
  inset: 0;
  z-index: 999;
  background: transparent;
}

.word-popover {
  position: fixed;
  z-index: 1000;
  background: rgba(var(--v-theme-surface), 0.95);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 12px;
  padding: 12px 16px;
  min-width: 160px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  transform: translate(-50%, 8px);
}

.word-popover__word {
  font-size: 1.1em;
  padding-right: 20px;
}

.word-popover__phonemes {
  color: rgb(var(--v-theme-primary));
  letter-spacing: 0.5px;
}

.word-popover__close {
  position: absolute;
  top: 4px;
  right: 4px;
}
</style>
