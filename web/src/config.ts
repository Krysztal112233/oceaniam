export const appConfig = {
  apiBaseUrl: (import.meta.env.VITE_API_BASE_URL || "/api").replace(/\/$/, ""),
  refreshWindowSeconds: 5 * 60,
} as const;
