import { defineStore } from "pinia";
import { ref } from "vue";

const AUTH_TOKEN_COOKIE_NAME = "auth_token";

function decodeJwtExp(token: string): number | null {
    const parts = token.split(".");
    if (parts.length < 2) return null;

    try {
        const payload = parts[1]!;
        const padded = payload.replace(/-/g, "+").replace(/_/g, "/");
        const json = atob(padded);
        const claims = JSON.parse(json) as { exp?: unknown };
        if (typeof claims.exp === "number") return claims.exp;
        if (typeof claims.exp === "string") {
            const n = Number(claims.exp);
            return Number.isFinite(n) ? n : null;
        }
        return null;
    } catch {
        return null;
    }
}

function isJwtExpired(token: string): boolean {
    const exp = decodeJwtExp(token);
    if (exp === null) return false;
    const now = Math.floor(Date.now() / 1000);
    return now >= exp;
}

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

        let validJwt: string | null = null;
        let initialExpiresAt: number | null = null;

        if (initialJwt) {
            if (isJwtExpired(initialJwt)) {
                writeAuthCookie(null);
            } else {
                validJwt = initialJwt;
                initialExpiresAt = decodeJwtExp(initialJwt);
            }
        }

        const isLoggedIn = ref(Boolean(validJwt));
        const username = ref<string | null>(null);
        const jwt = ref<string | null>(validJwt);
        const expiresAt = ref<number | null>(initialExpiresAt);

        function syncFromCookie(): void {
            const token = readCookie(AUTH_TOKEN_COOKIE_NAME);
            jwt.value = token;
            isLoggedIn.value = Boolean(token);
            if (!token) {
                username.value = null;
                expiresAt.value = null;
            } else {
                expiresAt.value = decodeJwtExp(token);
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
