<script setup lang="ts">
// Dev / visual-regression aid — exposed at `/_ui`. Not part of the
// product surface. We enumerate every generic component here so eyeball
// tests across a Tailwind upgrade, token tweak, or a11y audit stay
// cheap.
import { ref } from 'vue';
import Button from '@/components/Button.vue';
import Card from '@/components/Card.vue';
import Stat from '@/components/Stat.vue';
import PlayerChip from '@/components/PlayerChip.vue';
import CurrencyInput from '@/components/CurrencyInput.vue';
import TextField from '@/components/TextField.vue';
import BottomSheet from '@/components/BottomSheet.vue';
import LeaderboardRow from '@/components/LeaderboardRow.vue';
import StatusPill from '@/components/StatusPill.vue';
import { useToast } from '@/composables/useToast';
import { useConfirm } from '@/composables/useConfirm';
import { useMoney } from '@/composables/useMoney';

const { show } = useToast();
const { confirm } = useConfirm();
const money = useMoney();

const amount = ref<number>(1250);
const text = ref('');
const sheetOpen = ref(false);

async function openConfirm(danger = false) {
  const ok = await confirm({
    title: danger ? 'Night löschen?' : 'Sicher?',
    message: danger
      ? 'Diese Night und alle Buy-ins werden unwiderruflich entfernt.'
      : 'Nur eine Demo — klick einfach durch.',
    confirmText: danger ? 'Löschen' : 'Ja',
    cancelText: 'Abbrechen',
    danger,
  });
  show({
    message: ok ? 'Bestätigt' : 'Abgebrochen',
    variant: ok ? 'success' : 'info',
  });
}
</script>

<template>
  <main class="min-h-screen bg-bg text-text p-6 space-y-8 max-w-4xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold">UI demo</h1>
      <p class="text-text-muted text-sm">
        Dev-only showcase of every generic component.
        Owned by Frontend-Shell.
      </p>
    </header>

    <Card title="Buttons">
      <div class="flex flex-wrap gap-3">
        <Button variant="primary">Primary</Button>
        <Button variant="ghost">Ghost</Button>
        <Button variant="danger">Danger</Button>
        <Button variant="primary" loading>Loading</Button>
        <Button variant="primary" disabled>Disabled</Button>
        <Button variant="icon" aria-label="Hinzufügen">+</Button>
      </div>
    </Card>

    <Card title="Stats">
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <Stat label="Gewinn" :value="money.format(12450)" sign="positive" />
        <Stat label="Verlust" :value="money.format(-8900)" sign="negative" />
        <Stat label="Saldo" :value="money.format(0)" sign="neutral" />
      </div>
    </Card>

    <Card title="Player chips">
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <PlayerChip name="Anton Wiblishauser" subtitle="Admin" />
        <PlayerChip name="Max Mustermann" subtitle="Player" />
        <PlayerChip name="Petra Schulz" />
        <PlayerChip name="Li Wei" />
      </div>
    </Card>

    <Card title="Inputs">
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <TextField v-model="text" label="Name" placeholder="Anton" />
        <TextField
          v-model="text"
          label="E-Mail"
          type="email"
          error="Ungültige E-Mail"
        />
        <CurrencyInput v-model="amount" label="Buy-in" helper="Betrag in Euro" />
        <div class="flex items-end">
          <p class="text-sm text-text-muted">
            Current cents: <span class="tabular font-semibold">{{ amount }}</span>
          </p>
        </div>
      </div>
    </Card>

    <Card title="Status pills">
      <div class="flex gap-3">
        <StatusPill status="open" />
        <StatusPill status="closed" />
      </div>
    </Card>

    <Card title="Leaderboard rows">
      <ul>
        <LeaderboardRow
          :rank="1"
          name="Anton Wiblishauser"
          :net-cents="23450"
          :nights-played="7"
        />
        <LeaderboardRow
          :rank="2"
          name="Max Mustermann"
          :net-cents="1240"
          :nights-played="5"
        />
        <LeaderboardRow
          :rank="3"
          name="Petra Schulz"
          :net-cents="-9800"
          :nights-played="6"
        />
      </ul>
    </Card>

    <Card title="Bottom sheet">
      <div class="flex flex-col gap-3">
        <Button variant="primary" @click="sheetOpen = true">Sheet öffnen</Button>
      </div>
      <BottomSheet
        :open="sheetOpen"
        title="Buy-in hinzufügen"
        @update:open="sheetOpen = $event"
      >
        <form class="flex flex-col gap-4 pb-2" @submit.prevent="sheetOpen = false">
          <CurrencyInput v-model="amount" label="Betrag" />
          <TextField v-model="text" label="Notiz" />
          <Button type="submit" variant="primary">Speichern</Button>
        </form>
      </BottomSheet>
    </Card>

    <Card title="Confirm dialog">
      <div class="flex gap-3">
        <Button variant="ghost" @click="openConfirm(false)">Bestätigen</Button>
        <Button variant="danger" @click="openConfirm(true)">Gefährlich</Button>
      </div>
    </Card>

    <Card title="Toasts">
      <div class="flex flex-wrap gap-3">
        <Button variant="ghost" @click="show({ message: 'Info-Meldung' })">
          Info
        </Button>
        <Button
          variant="ghost"
          @click="show({ message: 'Gespeichert', variant: 'success' })"
        >
          Success
        </Button>
        <Button
          variant="ghost"
          @click="show({ message: 'Fast abgelaufen', variant: 'warning' })"
        >
          Warning
        </Button>
        <Button
          variant="ghost"
          @click="show({ message: 'Das ging schief', variant: 'error' })"
        >
          Error
        </Button>
      </div>
    </Card>
  </main>
</template>
