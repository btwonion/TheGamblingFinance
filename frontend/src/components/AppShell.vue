<script setup lang="ts">
import TopBar from './TopBar.vue';
import SideNav from './SideNav.vue';
import BottomNav from './BottomNav.vue';
</script>

<template>
  <div class="min-h-screen bg-bg text-text flex flex-col">
    <TopBar />
    <div class="flex flex-1 min-h-0">
      <SideNav />
      <!-- `pb-16` on mobile to leave room for the bottom nav (h-14 + border). -->
      <main class="flex-1 min-w-0 pb-16 md:pb-0">
        <slot />
      </main>
    </div>
    <BottomNav />
  </div>
</template>
