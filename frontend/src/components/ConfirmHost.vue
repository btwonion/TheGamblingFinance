<script setup lang="ts">
import ConfirmDialog from './ConfirmDialog.vue';
import { useConfirm } from '@/composables/useConfirm';

// Single host mounted in `App.vue`. Any code can call
// `useConfirm().confirm(...)` without having to render its own dialog.
const { state, resolve } = useConfirm();
</script>

<template>
  <ConfirmDialog
    :open="state.open"
    :title="state.title"
    :message="state.message"
    :confirm-text="state.confirmText"
    :cancel-text="state.cancelText"
    :danger="state.danger"
    @confirm="resolve(true)"
    @cancel="resolve(false)"
  />
</template>
