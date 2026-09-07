import {
  VueQueryPlugin,
  type VueQueryPluginOptions,
} from "@tanstack/vue-query";
import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "@/App.vue";
import { i18n } from "@/i18n";
import { router } from "@/router";
import { useAuthStore } from "@/stores/auth";
import { usePreferencesStore } from "@/stores/preferences";
import "@/icons";
import "@/style.css";

const app = createApp(App);
const pinia = createPinia();
const queryOptions: VueQueryPluginOptions = {
  queryClientConfig: {
    defaultOptions: {
      queries: { retry: 1, staleTime: 15_000, refetchOnWindowFocus: false },
      mutations: { retry: 0 },
    },
  },
};

app.use(pinia);
app.use(i18n);
app.use(VueQueryPlugin, queryOptions);
app.use(router);

usePreferencesStore();
await useAuthStore().initialize();
await router.isReady();
app.mount("#app");
