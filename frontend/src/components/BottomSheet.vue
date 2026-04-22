<script setup lang="ts">
import { nextTick, ref, watch, computed } from 'vue';

interface Props {
  open: boolean;
  title?: string;
  /** Max drag distance at which the sheet dismisses. */
  dismissThreshold?: number;
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  dismissThreshold: 80,
});

const emit = defineEmits<{ (e: 'update:open', value: boolean): void }>();

const sheetRef = ref<HTMLElement | null>(null);
const panelRef = ref<HTMLElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

// ------------------------------------------------------------------
// Focus trap + focus restore.
// ------------------------------------------------------------------

function getFocusable(root: HTMLElement | null): HTMLElement[] {
  if (!root) return [];
  const sel = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(',');
  return Array.from(root.querySelectorAll<HTMLElement>(sel)).filter(
    (el) => !el.hasAttribute('aria-hidden'),
  );
}

function onKeydown(ev: KeyboardEvent) {
  if (!props.open) return;
  if (ev.key === 'Escape') {
    ev.preventDefault();
    close();
    return;
  }
  if (ev.key !== 'Tab') return;
  const focusables = getFocusable(panelRef.value);
  if (focusables.length === 0) {
    ev.preventDefault();
    return;
  }
  const first = focusables[0];
  const last = focusables[focusables.length - 1];
  const active = document.activeElement as HTMLElement | null;
  if (ev.shiftKey) {
    if (active === first || !panelRef.value?.contains(active)) {
      ev.preventDefault();
      last.focus();
    }
  } else {
    if (active === last) {
      ev.preventDefault();
      first.focus();
    }
  }
}

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      previouslyFocused = document.activeElement as HTMLElement | null;
      await nextTick();
      const focusables = getFocusable(panelRef.value);
      (focusables[0] ?? panelRef.value)?.focus();
      document.addEventListener('keydown', onKeydown);
      document.body.style.overflow = 'hidden';
    } else {
      document.removeEventListener('keydown', onKeydown);
      document.body.style.overflow = '';
      previouslyFocused?.focus?.();
      previouslyFocused = null;
      translateY.value = 0;
    }
  },
);

// ------------------------------------------------------------------
// Drag-to-dismiss. Pointer events only — works for mouse + touch.
// ------------------------------------------------------------------

const dragging = ref(false);
const translateY = ref(0);
let startY = 0;

function onPointerDown(ev: PointerEvent) {
  dragging.value = true;
  startY = ev.clientY;
  (ev.target as HTMLElement).setPointerCapture?.(ev.pointerId);
}

function onPointerMove(ev: PointerEvent) {
  if (!dragging.value) return;
  const delta = ev.clientY - startY;
  translateY.value = Math.max(0, delta);
}

function onPointerUp() {
  if (!dragging.value) return;
  dragging.value = false;
  if (translateY.value > props.dismissThreshold) {
    close();
  } else {
    translateY.value = 0;
  }
}

function close() {
  emit('update:open', false);
}

const panelStyle = computed(() => ({
  transform: `translateY(${translateY.value}px)`,
  transition: dragging.value ? 'none' : undefined,
}));
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
        ref="sheetRef"
        class="fixed inset-0 z-50 bg-black/60"
        @click.self="close"
      >
        <Transition
          enter-active-class="transition-transform duration-med ease-out"
          leave-active-class="transition-transform duration-med ease-out"
          enter-from-class="translate-y-full"
          leave-to-class="translate-y-full"
          appear
        >
          <div
            v-if="open"
            ref="panelRef"
            role="dialog"
            aria-modal="true"
            :aria-label="title || 'Bottom sheet'"
            tabindex="-1"
            class="fixed inset-x-0 bottom-0 bg-surface rounded-t-xl shadow-sheet pt-2 pb-6 max-h-[85vh] overflow-y-auto"
            :style="panelStyle"
          >
            <!-- Drag handle -->
            <button
              type="button"
              class="mx-auto mt-1 mb-3 block w-12 h-1.5 rounded-full bg-border hover:bg-border-strong cursor-grab touch-none"
              aria-label="Schließen ziehen"
              @pointerdown="onPointerDown"
              @pointermove="onPointerMove"
              @pointerup="onPointerUp"
              @pointercancel="onPointerUp"
              @keydown.enter.prevent="close"
              @keydown.space.prevent="close"
            />
            <header v-if="title" class="px-4 md:px-6 pb-3">
              <h2 class="text-lg font-semibold text-text">{{ title }}</h2>
            </header>
            <div class="px-4 md:px-6">
              <slot />
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
