<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import Card from '@/components/Card.vue';
import TextField from '@/components/TextField.vue';
import Button from '@/components/Button.vue';
import { useAuthStore } from '@/stores/auth';

const auth = useAuthStore();
const router = useRouter();
const route = useRoute();

const email = ref('');
const password = ref('');

onMounted(() => {
  if (auth.isAuthed) {
    redirectAfterLogin();
  }
});

function redirectAfterLogin() {
  const next = (route.query.next as string) || '/';
  // Don't bounce back to the login page via `next=/login`.
  router.replace(next.startsWith('/login') ? '/' : next);
}

async function onSubmit() {
  const ok = await auth.login(email.value, password.value);
  if (ok) {
    redirectAfterLogin();
  }
}
</script>

<template>
  <main
    class="min-h-screen bg-bg text-text flex items-center justify-center p-4"
  >
    <Card class="w-full max-w-md" title="Anmelden">
      <form class="flex flex-col gap-4" novalidate @submit.prevent="onSubmit">
        <TextField
          v-model="email"
          label="E-Mail"
          type="email"
          autocomplete="username"
          required
          :error="''"
        />
        <TextField
          v-model="password"
          label="Passwort"
          type="password"
          autocomplete="current-password"
          required
        />

        <div
          v-if="auth.error"
          role="alert"
          class="text-sm text-negative bg-negative-soft border border-negative rounded-md px-3 py-2"
        >
          {{ auth.error }}
        </div>

        <Button
          type="submit"
          variant="primary"
          :loading="auth.loading"
          :disabled="auth.loading"
        >
          Anmelden
        </Button>
      </form>
    </Card>
  </main>
</template>
