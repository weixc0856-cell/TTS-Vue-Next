<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  current: number;
  total: number;
}>();

const percent = computed(() => {
  if (props.total === 0) return 0;
  return Math.round((props.current / props.total) * 100);
});
</script>

<template>
  <div class="session-progress">
    <div class="d-flex align-center justify-space-between mb-1">
      <span class="text-caption text-medium-emphasis">
        {{ $t('practice.shadowing.sentence') }} {{ current }}/{{ total }}
      </span>
      <span class="text-caption text-medium-emphasis">{{ percent }}%</span>
    </div>
    <v-progress-linear
      :model-value="percent"
      color="primary"
      height="4"
      rounded />
  </div>
</template>
