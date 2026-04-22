<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  name: string;
  subtitle?: string;
  /** Overrides the default two-letter initial. */
  initials?: string;
  /** Fallback avatar size — we design around 32 px. */
  size?: 'sm' | 'md';
}

const props = withDefaults(defineProps<Props>(), {
  subtitle: '',
  initials: undefined,
  size: 'md',
});

function makeInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return '??';
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

// Stable hash → one of a small token-defined palette. We lean on the
// semantic soft-background tokens so we don't need to introduce new hex.
const palette = [
  'bg-positive-soft text-positive',
  'bg-warning-soft text-warning',
  'bg-negative-soft text-negative',
  'bg-surface-2 text-text',
];

function hashName(name: string): number {
  let h = 0;
  for (let i = 0; i < name.length; i++) {
    h = (h * 31 + name.charCodeAt(i)) | 0;
  }
  return Math.abs(h);
}

const paletteClass = computed(() => palette[hashName(props.name) % palette.length]);
const initialsText = computed(() => (props.initials ?? makeInitials(props.name)));
const sizeClass = computed(() =>
  props.size === 'sm' ? 'w-8 h-8 text-xs' : 'w-10 h-10 text-sm',
);
</script>

<template>
  <div class="flex items-center gap-3 min-h-11">
    <div
      :class="[
        'rounded-full flex items-center justify-center font-semibold',
        paletteClass,
        sizeClass,
      ]"
      aria-hidden="true"
    >
      {{ initialsText }}
    </div>
    <div class="flex flex-col min-w-0">
      <span class="text-text font-medium truncate">{{ name }}</span>
      <span v-if="subtitle" class="text-xs text-text-muted truncate">{{ subtitle }}</span>
    </div>
  </div>
</template>
