import { createOceanIamClient } from "@oceaniam/sdk";

import { appConfig } from "@/config";
import { useAuthStore } from "@/stores/auth";

export const AUTH_HEADER = { Authorization: "" } as const;
export const APPLICATION_AUTH_HEADER = {
  Authorization: "",
  "X-OceanIAM-Application-Secret": "",
} as const;

export const api = createOceanIamClient({
  baseUrl: appConfig.apiBaseUrl,
  getToken: () => useAuthStore().token,
  beforeRequest: () => useAuthStore().ensureFreshToken(),
  onUnauthorized: async () => {
    const nextToken = await useAuthStore().handleUnauthorized();
    if (!nextToken && window.location.pathname !== "/login") {
      const redirect = `${window.location.pathname}${window.location.search}${window.location.hash}`;
      window.location.assign(`/login?redirect=${encodeURIComponent(redirect)}`);
    }
    return nextToken;
  },
});
