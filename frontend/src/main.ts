import { createApp } from 'vue';
import { createPinia } from 'pinia';

import App from './App.vue';
import { router } from './router';
import './styles/index.css';

// App bootstrap. Order matters: Pinia must be installed before any
// store is used, and the router before <RouterView> renders.
const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount('#app');
