import { reactive } from "vue";
import { doSystemAuth, doSystemSignout } from "@oceaniam/sdk";
import { useAuthStore } from "../stores/auth";
import { appConfig } from "../config";

export type AuthUser = {
    username: string;
    displayName: string;
};

export const authState = reactive<{
    user: AuthUser | null;
    loading: boolean;
    error: string | null;
}>({
    user: null,
    loading: false,
    error: null,
});

export async function login(username: string, password: string): Promise<void> {
    const normalizedUsername = username.trim();
    if (!normalizedUsername) {
        authState.error = "Username is required.";
        throw new Error(authState.error);
    }
    if (!password.trim()) {
        authState.error = "Password is required.";
        throw new Error(authState.error);
    }

    const authStore = useAuthStore();

    authState.loading = true;
    authState.error = null;

    try {
        const resp = await doSystemAuth(
            normalizedUsername,
            password,
            appConfig.systemBaseUrl,
            "Both",
        );

        // Prefer the server-issued auth cookie; use JSON jwt only as a fallback.
        authStore.syncFromCookie();
        if (!authStore.jwt && resp.jwt?.trim()) {
            authStore.setAuthToken(resp.jwt);
        }

        authStore.isLoggedIn = Boolean(authStore.jwt);
        authStore.username = normalizedUsername;
        authStore.expiresAt = null;

        authState.user = {
            username: normalizedUsername,
            displayName: normalizedUsername,
        };
    } catch (error) {
        authStore.isLoggedIn = false;
        authStore.username = null;
        authStore.expiresAt = null;
        authState.user = null;
        authState.error =
            error instanceof Error ? error.message : "Login failed.";
        throw error;
    } finally {
        authState.loading = false;
    }
}

export async function logout(): Promise<void> {
    const authStore = useAuthStore();
    const jwt = authStore.jwt;

    authState.loading = true;
    authState.error = null;

    try {
        if (jwt?.trim()) {
            await doSystemSignout(jwt, appConfig.systemBaseUrl);
        }
    } catch {
        // Keep local logout deterministic even if remote revocation fails.
    } finally {
        authStore.setAuthToken(null);
        authStore.isLoggedIn = false;
        authStore.username = null;
        authStore.expiresAt = null;

        authState.user = null;
        authState.loading = false;
    }
}
