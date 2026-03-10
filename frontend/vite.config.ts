/// <reference types="node" />

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";

// https://vite.dev/config/
export default defineConfig(() => {
    return {
        plugins: [tailwindcss(), vue()],
    };
});
