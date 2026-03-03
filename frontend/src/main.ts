import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import "@fontsource-variable/noto-sans-sc/index.css";
import { router } from "./router";
import { createPinia } from "pinia";

const pinia = createPinia();

createApp(App).use(router).use(pinia).mount("#app");
