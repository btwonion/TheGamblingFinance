<script setup lang="ts">
import { computed, ref, watch, useId } from 'vue';
import { useMoney } from '@/composables/useMoney';

/**
 * `CurrencyInput` — accepts / emits **integer cents**.
 *
 * Users type freely (`12,50`, `1234`, `1.234,56 €`). On blur we reformat
 * to canonical `de-DE` currency. Nothing in this component ever holds a
 * float representation — we parse to cents and reformat from cents.
 */

interface Props {
  modelValue: number;
  label?: string;
  error?: string;
  helper?: string;
  placeholder?: string;
  name?: string;
  allowNegative?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  label: '',
  error: '',
  helper: '',
  placeholder: '0,00 €',
  name: undefined,
  allowNegative: false,
});

const emit = defineEmits<{
  (e: 'update:modelValue', cents: number): void;
  (e: 'blur', ev: FocusEvent): void;
}>();

const money = useMoney();
const id = (typeof useId === 'function' ? useId() : `ci-${Math.random().toString(36).slice(2)}`);

// What the user sees in the <input>. Kept separate from the integer
// cents value; only the cents value is the source of truth.
const display = ref(props.modelValue === 0 ? '' : money.format(props.modelValue));
const focused = ref(false);

// If the parent changes the value out from under us, update display —
// but not while the user is typing.
watch(
  () => props.modelValue,
  (next) => {
    if (focused.value) return;
    display.value = next === 0 ? '' : money.format(next);
  },
);

function onInput(ev: Event) {
  const raw = (ev.target as HTMLInputElement).value;
  display.value = raw;
  const cents = money.parse(raw);
  const emitted = props.allowNegative ? cents : Math.max(0, cents);
  emit('update:modelValue', emitted);
}

function onFocus() {
  focused.value = true;
}

function onBlur(ev: FocusEvent) {
  focused.value = false;
  // Reformat on blur for a consistent canonical display.
  display.value = props.modelValue === 0 ? '' : money.format(props.modelValue);
  emit('blur', ev);
}

const inputClasses = computed(() => {
  const base =
    'block w-full h-11 px-3 rounded-md bg-surface-2 text-text placeholder:text-text-dim tabular ' +
    'border border-border transition-colors duration-fast text-right ' +
    'focus:outline-none focus:border-border-strong focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus';
  const err = props.error ? 'border-negative focus:border-negative' : '';
  return [base, 'text-base', err].join(' ');
});
</script>

<template>
  <div class="flex flex-col gap-1.5">
    <label v-if="label" :for="id" class="text-sm text-text-muted font-medium">
      {{ label }}
    </label>
    <input
      :id="id"
      :name="name"
      type="text"
      inputmode="decimal"
      :value="display"
      :placeholder="placeholder"
      :aria-invalid="!!error || undefined"
      :aria-describedby="error ? `${id}-err` : helper ? `${id}-help` : undefined"
      :class="inputClasses"
      @input="onInput"
      @focus="onFocus"
      @blur="onBlur"
    />
    <p v-if="error" :id="`${id}-err`" class="text-xs text-negative" role="alert">
      {{ error }}
    </p>
    <p v-else-if="helper" :id="`${id}-help`" class="text-xs text-text-muted">
      {{ helper }}
    </p>
  </div>
</template>
