<script setup lang="ts">
import Card from '@/components/Card.vue';
import Button from '@/components/Button.vue';
import { useRouter } from 'vue-router';
const router = useRouter();
</script>

<template>
  <main class="min-h-screen bg-bg text-text flex items-center justify-center p-6">
    <Card class="max-w-md w-full" title="Seite nicht gefunden">
      <p class="text-text-muted mb-4">
        Die angeforderte Seite existiert nicht.
      </p>
      <Button variant="primary" @click="router.push('/')">Zur Startseite</Button>
    </Card>
  </main>
</template>
