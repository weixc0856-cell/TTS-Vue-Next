<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    audioLevel?: number;
    isActive?: boolean;
    bars?: number;
    height?: number;
    color?: string;
  }>(),
  {
    audioLevel: 0,
    isActive: false,
    bars: 32,
    height: 48,
    color: 'rgb(var(--v-theme-primary))',
  },
);

const canvasRef = ref<HTMLCanvasElement | null>(null);
let animationId: number | null = null;
let barValues: number[] = [];

function initBars() {
  barValues = Array.from({ length: props.bars }, () => 0);
}

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const width = canvas.width;
  const height = canvas.height;

  ctx.clearRect(0, 0, width, height);

  const barWidth = width / props.bars - 2;
  const gap = 2;

  // Shift bars and add new value at front
  if (props.isActive) {
    barValues.pop();
    barValues.unshift(Math.min(props.audioLevel, 1));
  } else {
    // Decay to silence
    barValues = barValues.map((v) => Math.max(0, v - 0.02));
  }

  for (let i = 0; i < props.bars; i++) {
    const barHeight = barValues[i] * height;
    const x = i * (barWidth + gap);
    const y = height - barHeight;

    const gradient = ctx.createLinearGradient(0, y, 0, height);
    gradient.addColorStop(0, props.color);
    gradient.addColorStop(1, 'rgba(var(--v-theme-primary), 0.3)');

    ctx.fillStyle = gradient;
    ctx.fillRect(x, y, barWidth, barHeight);
  }

  animationId = requestAnimationFrame(draw);
}

onMounted(() => {
  initBars();
  if (canvasRef.value) {
    const dpr = window.devicePixelRatio || 1;
    canvasRef.value.width = canvasRef.value.clientWidth * dpr;
    canvasRef.value.height = props.height * dpr;
    canvasRef.value
      .getContext('2d')
      ?.scale(dpr, dpr);
  }
  draw();
});

onUnmounted(() => {
  if (animationId) {
    cancelAnimationFrame(animationId);
  }
});

watch(
  () => props.isActive,
  (active) => {
    if (!active) {
      // Bars will decay naturally
    }
  },
);
</script>

<template>
  <canvas
    ref="canvasRef"
    class="audio-visualizer"
    :style="{ width: '100%', height: `${height}px` }" />
</template>

<style scoped>
.audio-visualizer {
  display: block;
  border-radius: 8px;
  background: rgba(var(--v-theme-on-surface), 0.03);
}
</style>
