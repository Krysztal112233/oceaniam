import { defineStore } from "pinia";
import { ref } from "vue";

const AUTH_TOKEN_COOKIE_NAME = "auth_token";

// NOTE: AI-generated function
function readCookie(name: string): string | null {
    if (typeof document === "undefined") return null;

    const key = `${encodeURIComponent(name)}=`;
    const all = document.cookie ? document.cookie.split(";") : [];
    for (const item of all) {
        const part = item.trim();
        if (!part.startsWith(key)) continue;

        const raw = part.slice(key.length);
        try {
            return decodeURIComponent(raw);
        } catch {
            return raw;
        }
    }

    return null;
}

// NOTE: AI-generated function
function writeAuthCookie(token: string | null): void {
    if (typeof document === "undefined") return;

    const normalized = token?.trim();
    if (!normalized) {
        document.cookie = `${AUTH_TOKEN_COOKIE_NAME}=; Max-Age=0; Path=/`;
        return;
    }

    document.cookie = `${AUTH_TOKEN_COOKIE_NAME}=${encodeURIComponent(normalized)}; Path=/`;
}

export const useAuthStore = defineStore("auth", {
    state: () => {
        const initialJwt = readCookie(AUTH_TOKEN_COOKIE_NAME);

        const isLoggedIn = ref(Boolean(initialJwt));
        const username = ref<string | null>(null);
        const jwt = ref<string | null>(initialJwt);
        const expiresAt = ref<number | null>(null);

        function syncFromCookie(): void {
            const token = readCookie(AUTH_TOKEN_COOKIE_NAME);
            jwt.value = token;
            isLoggedIn.value = Boolean(token);
            if (!token) {
                username.value = null;
                expiresAt.value = null;
            }
        }

        function setAuthToken(token: string | null): void {
            writeAuthCookie(token);
            syncFromCookie();
        }

        return {
            isLoggedIn,
            expiresAt,
            username,
            jwt,
            syncFromCookie,
            setAuthToken,
        };
    },
    persist: true,
});
