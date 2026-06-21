import { defineStore } from "pinia";

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

function writeAuthCookie(token: string | null): void {
    if (typeof document === "undefined") return;

    const normalized = token?.trim();
    if (!normalized) {
        document.cookie = `${AUTH_TOKEN_COOKIE_NAME}=; Max-Age=0; Path=/; Secure; SameSite=Lax`;
        return;
    }

    document.cookie = `${AUTH_TOKEN_COOKIE_NAME}=${encodeURIComponent(normalized)}; Path=/; Secure; SameSite=Lax`;
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

        return {
            isLoggedIn: Boolean(validJwt),
            username: null as string | null,
            jwt: validJwt,
            expiresAt: initialExpiresAt,
        };
    },
    actions: {
        syncFromCookie(): void {
            const token = readCookie(AUTH_TOKEN_COOKIE_NAME);
            this.jwt = token;
            this.isLoggedIn = Boolean(token);
            if (!token) {
                this.username = null;
                this.expiresAt = null;
            } else {
                this.expiresAt = decodeJwtExp(token);
            }
        },
        setAuthToken(token: string | null): void {
            writeAuthCookie(token);
            this.jwt = token;
            this.isLoggedIn = Boolean(token);
            if (!token) {
                this.username = null;
                this.expiresAt = null;
            } else {
                this.expiresAt = decodeJwtExp(token);
            }
        },
    },
    persist: true,
});
