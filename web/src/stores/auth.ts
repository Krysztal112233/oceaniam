import {
  createOceanIamClient,
  OceanIamApiError,
  systemRefresh,
  systemSignin,
  systemSignout,
  unwrap,
  type components,
} from "@oceaniam/sdk";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { appConfig } from "@/config";
import { tokenNeedsRefresh } from "@/utils/jwt";

type AdministratorProfile = components["schemas"]["AdministratorProfileVO"];

const TOKEN_KEY = "oceaniam.auth.token";
const AUTH_PLACEHOLDER = { Authorization: "" } as const;

export const useAuthStore = defineStore("auth", () => {
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY));
  const profile = ref<AdministratorProfile | null>(null);
  const ready = ref(false);
  const busy = ref(false);
  const error = ref<string | null>(null);
  let refreshPromise: Promise<string | null> | null = null;

  const isAuthenticated = computed(() => Boolean(token.value && profile.value));

  function saveToken(value: string | null) {
    token.value = value;
    if (value) localStorage.setItem(TOKEN_KEY, value);
    else localStorage.removeItem(TOKEN_KEY);
  }

  function clearSession(message: string | null = null) {
    saveToken(null);
    profile.value = null;
    error.value = message;
  }

  function needsRefresh(value: string): boolean {
    return tokenNeedsRefresh(value, appConfig.refreshWindowSeconds);
  }

  async function loadProfile(value: string): Promise<AdministratorProfile> {
    const client = createOceanIamClient({
      baseUrl: appConfig.apiBaseUrl,
      getToken: () => value,
    });
    return unwrap(
      await client.GET("/administrators/me", {
        params: { header: AUTH_PLACEHOLDER },
      }),
    );
  }

  async function performRefresh(expectedToken: string): Promise<string | null> {
    const latest = localStorage.getItem(TOKEN_KEY);
    if (latest && latest !== expectedToken && !needsRefresh(latest)) {
      token.value = latest;
      return latest;
    }

    const source = latest || expectedToken;
    try {
      const next = await systemRefresh(appConfig.apiBaseUrl, source);
      saveToken(next);
      return next;
    } catch {
      clearSession();
      return null;
    }
  }

  async function coordinatedRefresh(
    expectedToken: string,
  ): Promise<string | null> {
    if (typeof navigator !== "undefined" && navigator.locks) {
      return navigator.locks.request("oceaniam.auth.refresh", () =>
        performRefresh(expectedToken),
      );
    }
    return performRefresh(expectedToken);
  }

  async function refresh(force = false): Promise<string | null> {
    const current = token.value;
    if (!current) return null;
    if (!force && !needsRefresh(current)) return current;
    if (!refreshPromise) {
      refreshPromise = coordinatedRefresh(current).finally(() => {
        refreshPromise = null;
      });
    }
    return refreshPromise;
  }

  async function initialize() {
    if (ready.value) return;
    const current = token.value;
    if (!current) {
      ready.value = true;
      return;
    }

    busy.value = true;
    try {
      const fresh = await refresh();
      if (!fresh) return;
      profile.value = await loadProfile(fresh);
    } catch {
      clearSession();
    } finally {
      busy.value = false;
      ready.value = true;
    }
  }

  async function signin(name: string, password: string) {
    busy.value = true;
    error.value = null;
    try {
      const jwt = await systemSignin(
        appConfig.apiBaseUrl,
        name.trim(),
        password,
      );
      saveToken(jwt);
      profile.value = await loadProfile(jwt);
    } catch (cause) {
      clearSession(
        cause instanceof OceanIamApiError && cause.status === 401
          ? "invalid"
          : "failed",
      );
      throw cause;
    } finally {
      busy.value = false;
      ready.value = true;
    }
  }

  async function signout() {
    const current = token.value;
    busy.value = true;
    try {
      if (current) await systemSignout(appConfig.apiBaseUrl, current);
    } catch {
      // Local sign-out remains deterministic when revocation cannot be reached.
    } finally {
      clearSession();
      busy.value = false;
    }
  }

  window.addEventListener("storage", (event) => {
    if (event.key !== TOKEN_KEY) return;
    token.value = event.newValue;
    if (!event.newValue) {
      profile.value = null;
      if (window.location.pathname !== "/login") {
        const redirect = `${window.location.pathname}${window.location.search}${window.location.hash}`;
        window.location.assign(
          `/login?redirect=${encodeURIComponent(redirect)}`,
        );
      }
    }
  });

  return {
    busy,
    error,
    isAuthenticated,
    profile,
    ready,
    token,
    clearSession,
    ensureFreshToken: () => refresh(false),
    handleUnauthorized: () => refresh(true),
    initialize,
    signin,
    signout,
  };
});
