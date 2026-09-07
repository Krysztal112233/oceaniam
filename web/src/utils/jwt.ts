export function jwtExpiration(token: string): number | null {
  try {
    const payload = token.split(".")[1];
    if (!payload) return null;
    const normalized = payload.replace(/-/g, "+").replace(/_/g, "/");
    const decoded = JSON.parse(atob(normalized)) as { exp?: unknown };
    return typeof decoded.exp === "number" ? decoded.exp : null;
  } catch {
    return null;
  }
}

export function tokenNeedsRefresh(
  token: string,
  refreshWindowSeconds: number,
  nowSeconds = Math.floor(Date.now() / 1000),
): boolean {
  const expiration = jwtExpiration(token);
  return expiration === null || expiration - nowSeconds <= refreshWindowSeconds;
}
