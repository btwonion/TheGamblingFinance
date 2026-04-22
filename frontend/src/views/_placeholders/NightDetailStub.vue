<script setup lang="ts">
import Card from '@/components/Card.vue';
import { useRoute } from 'vue-router';
const route = useRoute();
</script>

<template>
  <section class="p-6">
    <Card title="Night detail">
      <p class="text-text-muted">
        Frontend-Features fills this in in Phase 2. Night id:
        <span class="tabular">{{ route.params.id }}</span>.
      </p>
    </Card>
  </section>
</template>
