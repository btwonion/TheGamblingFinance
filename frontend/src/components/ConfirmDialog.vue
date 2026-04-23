<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import Button from './Button.vue';

interface Props {
  open: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  confirmText: 'Bestätigen',
  cancelText: 'Abbrechen',
  danger: false,
});

const emit = defineEmits<{
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();

const panelRef = ref<HTMLElement | null>(null);
const confirmRef = ref<InstanceType<typeof Button> | null>(null);
let previouslyFocused: HTMLElement | null = null;

function onKeydown(ev: KeyboardEvent) {
  if (!props.open) return;
  if (ev.key === 'Escape') {
    ev.preventDefault();
    emit('cancel');
    return;
  }
  if (ev.key !== 'Tab') return;
  const root = panelRef.value;
  if (!root) return;
  const focusables = Array.from(
    root.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
    ),
  ).filter((el) => !el.hasAttribute('disabled'));
  if (focusables.length === 0) {
    ev.preventDefault();
    return;
  }
  const first = focusables[0];
  const last = focusables[focusables.length - 1];
  const active = document.activeElement as HTMLElement | null;
  if (ev.shiftKey) {
    if (active === first || !root.contains(active)) {
      ev.preventDefault();
      last.focus();
    }
  } else if (active === last) {
    ev.preventDefault();
    first.focus();
  }
}

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      previouslyFocused = document.activeElement as HTMLElement | null;
      await nextTick();
      // Default focus on the confirm button unless danger → cancel is safer.
      const target = (panelRef.value?.querySelector('button') as HTMLButtonElement) ?? null;
      target?.focus();
      document.addEventListener('keydown', onKeydown);
    } else {
      document.removeEventListener('keydown', onKeydown);
      previouslyFocused?.focus?.();
      previouslyFocused = null;
    }
  },
);
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-med ease-out"
      leave-active-class="transition-opacity duration-med ease-out"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="fixed inset-0 z-50 bg-black/60 flex items-end md:items-center justify-center p-4"
        @click.self="emit('cancel')"
      >
        <div
          ref="panelRef"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="'confirm-title'"
          :aria-describedby="'confirm-msg'"
          tabindex="-1"
          class="w-full max-w-md bg-surface border border-border rounded-lg shadow-2 p-6"
        >
          <h2 id="confirm-title" class="text-lg font-semibold text-text mb-2">
            {{ title }}
          </h2>
          <p id="confirm-msg" class="text-sm text-text-muted mb-6">
            {{ message }}
          </p>
          <div class="flex gap-3 justify-end">
            <Button variant="ghost" @click="emit('cancel')">{{ cancelText }}</Button>
            <Button
              ref="confirmRef"
              :variant="danger ? 'danger' : 'primary'"
              @click="emit('confirm')"
            >
              {{ confirmText }}
            </Button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
