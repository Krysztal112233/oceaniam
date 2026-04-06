import { OceanIamClient, doSystemRefreshToken } from "@oceaniam/sdk";
import { appConfig } from "../config";
import { useAuthStore } from "../stores/auth";

let refreshPromise: Promise<string | null> | null = null;

async function handleUnauthorized(): Promise<string | null> {
    if (!refreshPromise) {
        refreshPromise = doRefresh().finally(() => {
            refreshPromise = null;
        });
    }
    return refreshPromise;
}

async function doRefresh(): Promise<string | null> {
    const authStore = useAuthStore();
    const currentJwt = authStore.jwt;
    if (!currentJwt?.trim()) return null;

    try {
        await doSystemRefreshToken(currentJwt, appConfig.systemBaseUrl, "Both");

        authStore.syncFromCookie();
        if (!authStore.jwt) return null;

        authStore.isLoggedIn = true;
        return authStore.jwt;
    } catch {
        authStore.setAuthToken(null);
        authStore.isLoggedIn = false;
        authStore.username = null;
        authStore.expiresAt = null;
        return null;
    }
}

export function getClient(): OceanIamClient {
    const authStore = useAuthStore();

    return new OceanIamClient({
        baseUrl: appConfig.systemBaseUrl,
        tokenGetter: () => authStore.jwt,
        onUnauthorized: handleUnauthorized,
    });
}
