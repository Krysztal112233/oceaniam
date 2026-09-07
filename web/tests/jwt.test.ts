import { describe, expect, it } from "vitest";

import { jwtExpiration, tokenNeedsRefresh } from "@/utils/jwt";

function token(payload: object) {
  return `header.${btoa(JSON.stringify(payload)).replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_")}.signature`;
}

describe("JWT refresh timing", () => {
  it("extracts a base64url expiration", () => {
    expect(jwtExpiration(token({ exp: 12345 }))).toBe(12345);
  });

  it("refreshes inside the configured window", () => {
    expect(tokenNeedsRefresh(token({ exp: 1300 }), 300, 1000)).toBe(true);
    expect(tokenNeedsRefresh(token({ exp: 1301 }), 300, 1000)).toBe(false);
  });

  it("treats malformed tokens as requiring refresh", () => {
    expect(tokenNeedsRefresh("not-a-jwt", 300, 1000)).toBe(true);
  });
});
