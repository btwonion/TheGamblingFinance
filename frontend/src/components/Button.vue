<script setup lang="ts">
import { computed } from 'vue';

type Variant = 'primary' | 'ghost' | 'danger' | 'icon';

interface Props {
  variant?: Variant;
  type?: 'button' | 'submit' | 'reset';
  disabled?: boolean;
  loading?: boolean;
  ariaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  type: 'button',
  disabled: false,
  loading: false,
  ariaLabel: undefined,
});

defineEmits<{ (e: 'click', ev: MouseEvent): void }>();

const classes = computed(() => {
  const base =
    'inline-flex items-center justify-center gap-2 font-medium transition-colors duration-fast ease-out ' +
    'focus:outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus ' +
    'disabled:opacity-50 disabled:cursor-not-allowed select-none';

  const sized =
    props.variant === 'icon'
      ? 'w-11 h-11 min-w-11 min-h-11 rounded-full'
      : 'h-11 min-h-11 px-4 rounded-md';

  const variantClass = (() => {
    switch (props.variant) {
      case 'primary':
        return 'bg-primary text-primary-ink hover:bg-primary-hover active:bg-primary-active';
      case 'ghost':
        return 'bg-transparent text-text border border-border hover:bg-surface-2';
      case 'danger':
        return 'bg-transparent text-negative border border-negative hover:bg-negative-soft';
      case 'icon':
        return 'bg-surface-2 text-text hover:bg-surface border border-border';
      default:
        return '';
    }
  })();

  return [base, sized, variantClass].join(' ');
});
</script>

<template>
  <button
    :class="classes"
    :type="type"
    :disabled="disabled || loading"
    :aria-label="ariaLabel"
    :aria-busy="loading || undefined"
    @click="$emit('click', $event)"
  >
    <span
      v-if="loading"
      class="inline-block w-4 h-4 rounded-full border-2 border-current border-t-transparent animate-spin"
      aria-hidden="true"
    />
    <slot />
  </button>
</template>
