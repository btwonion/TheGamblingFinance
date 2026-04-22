<script setup lang="ts">
import { computed } from 'vue';

type Sign = 'positive' | 'negative' | 'neutral';

interface Props {
  label: string;
  value: string | number;
  unit?: string;
  sign?: Sign;
}

const props = withDefaults(defineProps<Props>(), {
  unit: '',
  sign: 'neutral',
});

const colorClass = computed(() => {
  switch (props.sign) {
    case 'positive':
      return 'text-positive';
    case 'negative':
      return 'text-negative';
    default:
      return 'text-text';
  }
});

const ariaLabel = computed(() => {
  const signWord =
    props.sign === 'positive' ? 'Plus' : props.sign === 'negative' ? 'Minus' : '';
  return `${props.label}: ${signWord} ${props.value}${props.unit ? ' ' + props.unit : ''}`.trim();
});
</script>

<template>
  <div
    class="flex flex-col gap-1"
    role="group"
    :aria-label="ariaLabel"
  >
    <span class="text-xs uppercase tracking-wider text-text-muted font-semibold">
      {{ label }}
    </span>
    <span class="text-2xl font-bold tabular" :class="colorClass">
      {{ value }}<span v-if="unit" class="text-base font-medium ml-1">{{ unit }}</span>
    </span>
  </div>
</template>
