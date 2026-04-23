<script setup lang="ts">
interface Props {
  title?: string;
  /** HTML tag for the heading (default `h3`). */
  titleAs?: 'h2' | 'h3' | 'h4';
}

withDefaults(defineProps<Props>(), {
  title: '',
  titleAs: 'h3',
});
</script>

<template>
  <section
    class="bg-surface border border-border rounded-lg shadow-1 p-4 md:p-6"
  >
    <component
      :is="titleAs"
      v-if="title || $slots.title"
      class="text-lg font-semibold text-text mb-3"
    >
      <slot name="title">{{ title }}</slot>
    </component>
    <slot />
    <div v-if="$slots.footer" class="mt-4 pt-4 border-t border-border">
      <slot name="footer" />
    </div>
  </section>
</template>
