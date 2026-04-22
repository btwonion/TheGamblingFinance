<script setup lang="ts">
import { computed } from 'vue';

type Status = 'open' | 'closed';

interface Props {
  status: Status;
}

const props = defineProps<Props>();

const classes = computed(() =>
  props.status === 'open'
    ? 'bg-positive-soft text-positive'
    : 'bg-surface-2 text-text-muted',
);

const label = computed(() => (props.status === 'open' ? 'Offen' : 'Abgeschlossen'));
</script>

<template>
  <span
    :class="[
      'inline-flex items-center px-2 py-0.5 rounded-md text-xs font-semibold uppercase tracking-wide',
      classes,
    ]"
  >
    {{ label }}
  </span>
</template>
