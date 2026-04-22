<script setup lang="ts">
import { computed, useId } from 'vue';

interface Props {
  modelValue: string;
  label?: string;
  type?: string;
  error?: string;
  helper?: string;
  autocomplete?: string;
  placeholder?: string;
  required?: boolean;
  name?: string;
  inputmode?: 'text' | 'numeric' | 'decimal' | 'email' | 'tel' | 'url' | 'search' | 'none';
}

const props = withDefaults(defineProps<Props>(), {
  label: '',
  type: 'text',
  error: '',
  helper: '',
  autocomplete: undefined,
  placeholder: '',
  required: false,
  name: undefined,
  inputmode: undefined,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'blur', ev: FocusEvent): void;
}>();

// Vue 3.5+ has `useId()`. We fall back for older envs just in case.
const id = (typeof useId === 'function' ? useId() : `tf-${Math.random().toString(36).slice(2)}`);

const inputClasses = computed(() => {
  const base =
    'block w-full h-11 px-3 rounded-md bg-surface-2 text-text placeholder:text-text-dim ' +
    'border border-border transition-colors duration-fast ' +
    'focus:outline-none focus:border-border-strong focus-visible:outline focus-visible:outline-2 focus-visible:outline-focus';
  // font-size is wired via the global reset; keep a belt-and-braces class.
  const size = 'text-base';
  const err = props.error ? 'border-negative focus:border-negative' : '';
  return [base, size, err].join(' ');
});
</script>

<template>
  <div class="flex flex-col gap-1.5">
    <label
      v-if="label"
      :for="id"
      class="text-sm text-text-muted font-medium"
    >
      {{ label }}
      <span v-if="required" aria-hidden="true" class="text-negative">*</span>
    </label>
    <input
      :id="id"
      :name="name"
      :type="type"
      :value="modelValue"
      :autocomplete="autocomplete"
      :placeholder="placeholder"
      :required="required"
      :inputmode="inputmode"
      :aria-invalid="!!error || undefined"
      :aria-describedby="error ? `${id}-err` : helper ? `${id}-help` : undefined"
      :class="inputClasses"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @blur="emit('blur', $event)"
    />
    <p
      v-if="error"
      :id="`${id}-err`"
      class="text-xs text-negative"
      role="alert"
    >
      {{ error }}
    </p>
    <p
      v-else-if="helper"
      :id="`${id}-help`"
      class="text-xs text-text-muted"
    >
      {{ helper }}
    </p>
  </div>
</template>
