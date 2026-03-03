import { defineStore } from "pinia";
import { ref } from "vue";

export const useAuthStore = defineStore("auth", () => {
    const isLoggedIn = ref(false);
    const username = ref<string | null>(null);
    const jwt = ref<string | null>(null);
    const expiresAt = ref<number | null>(null);

    return { isLoggedIn, expiresAt, username, jwt };
});
