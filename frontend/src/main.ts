import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import "@fontsource-variable/noto-sans-sc/index.css";
import { router } from "./router";

createApp(App).use(router).mount("#app");
