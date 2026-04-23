<script setup lang="ts">
import { computed } from 'vue';
import PlayerChip from './PlayerChip.vue';
import { useMoney } from '@/composables/useMoney';

interface Props {
  rank: number;
  name: string;
  netCents: number;
  nightsPlayed?: number;
}

const props = defineProps<Props>();

const money = useMoney();

const rankClass = computed(() =>
  props.rank === 1
    ? 'bg-warning-soft text-warning'
    : 'bg-surface-2 text-text-muted',
);

const amountClass = computed(() => {
  const s = money.sign(props.netCents);
  if (s === 'positive') return 'text-positive';
  if (s === 'negative') return 'text-negative';
  return 'text-text';
});

const amountLabel = computed(() => {
  const s = money.sign(props.netCents);
  const word = s === 'positive' ? 'Gewinn' : s === 'negative' ? 'Verlust' : 'Ausgeglichen';
  return `${word}: ${money.format(props.netCents)}`;
});
</script>

<template>
  <li class="flex items-center gap-3 min-h-12 py-2 border-b border-border last:border-b-0">
    <div
      :class="[
        'w-8 h-8 flex items-center justify-center rounded-full font-semibold text-sm tabular',
        rankClass,
      ]"
      aria-hidden="true"
    >
      {{ rank }}
    </div>
    <div class="flex-1 min-w-0">
      <PlayerChip
        :name="name"
        :subtitle="nightsPlayed !== undefined ? `${nightsPlayed} Nights` : undefined"
        size="sm"
      />
    </div>
    <span class="tabular font-semibold" :class="amountClass" :aria-label="amountLabel">
      {{ money.format(netCents) }}
    </span>
  </li>
</template>
