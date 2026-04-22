<script setup lang="ts">
import { computed } from 'vue';

type Variant = 'info' | 'success' | 'warning' | 'error';

interface Props {
  message: string;
  variant?: Variant;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'info',
});

const roleAttr = computed(() => (props.variant === 'error' ? 'alert' : 'status'));

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'success':
      return 'bg-positive-soft text-positive border-positive';
    case 'warning':
      return 'bg-warning-soft text-warning border-warning';
    case 'error':
      return 'bg-negative-soft text-negative border-negative';
    default:
      return 'bg-surface-2 text-text border-border';
  }
});
</script>

<template>
  <div
    :role="roleAttr"
    aria-live="polite"
    :class="[
      'px-4 py-3 rounded-md border shadow-2 text-sm max-w-sm',
      variantClasses,
    ]"
  >
    {{ message }}
  </div>
</template>
