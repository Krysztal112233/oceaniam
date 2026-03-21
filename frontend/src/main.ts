import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import "@fontsource-variable/noto-sans-sc/index.css";
import { router } from "./router";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import Toast from "vue-toastification";
import "vue-toastification/dist/index.css";

const pinia = createPinia().use(piniaPluginPersistedstate);

createApp(App).use(router).use(pinia).use(Toast).mount("#app");
