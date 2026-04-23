<script setup lang="ts">
import Toast from './Toast.vue';
import { useToast } from '@/composables/useToast';

const { toasts } = useToast();
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed z-50 bottom-24 md:bottom-6 right-4 flex flex-col gap-2 pointer-events-none"
    >
      <div v-for="t in toasts" :key="t.id" class="pointer-events-auto">
        <Toast :message="t.message" :variant="t.variant" />
      </div>
    </div>
  </Teleport>
</template>
