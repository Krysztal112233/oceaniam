import { createOceanIamClient } from "@oceaniam/sdk";
import { describe, expect, it, vi } from "vitest";

const AUTH_HEADER = { Authorization: "" } as const;

describe("OceanIAM API client", () => {
  it("refreshes once after the backend's 203 challenge", async () => {
    let token = "old-token";
    const seenTokens: Array<string | null> = [];
    const fetch = vi.fn((input: RequestInfo | URL) => {
      const request = input instanceof Request ? input : new Request(input);
      seenTokens.push(request.headers.get("Authorization"));
      if (seenTokens.length === 1) {
        return Promise.resolve(
          new Response(JSON.stringify({ msg: "expired" }), {
            status: 203,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            total_tenants: 0,
            total_applications: 0,
            total_administrators: 1,
            total_application_users: 0,
            total_active_secrets: 0,
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    });
    const refresh = vi.fn(() => {
      token = "new-token";
      return Promise.resolve(token);
    });
    const client = createOceanIamClient({
      baseUrl: "http://oceaniam.test",
      fetch,
      getToken: () => token,
      onUnauthorized: refresh,
    });

    const result = await client.GET("/statistics", {
      params: { header: AUTH_HEADER },
    });

    expect(result.response.status).toBe(200);
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(refresh).toHaveBeenCalledTimes(1);
    expect(seenTokens).toEqual(["Bearer old-token", "Bearer new-token"]);
  });

  it("never retries a second authentication failure", async () => {
    const fetch = vi.fn(() =>
      Promise.resolve(
        new Response(JSON.stringify({ msg: "authentication failed" }), {
          status: 400,
          headers: { "Content-Type": "application/json" },
        }),
      ),
    );
    const refresh = vi.fn(() => Promise.resolve("still-invalid"));
    const client = createOceanIamClient({
      baseUrl: "http://oceaniam.test",
      fetch,
      getToken: () => "old-token",
      onUnauthorized: refresh,
    });

    const result = await client.GET("/statistics", {
      params: { header: AUTH_HEADER },
    });

    expect(result.response.status).toBe(401);
    expect(result.data).toBeUndefined();
    expect(result.error).toEqual({ msg: "authentication failed" });
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("does not refresh for an ordinary validation error", async () => {
    const fetch = vi.fn(() =>
      Promise.resolve(
        new Response(JSON.stringify({ msg: "invalid query" }), {
          status: 400,
          headers: { "Content-Type": "application/json" },
        }),
      ),
    );
    const refresh = vi.fn(() => Promise.resolve("unused"));
    const client = createOceanIamClient({
      baseUrl: "http://oceaniam.test",
      fetch,
      getToken: () => "old-token",
      onUnauthorized: refresh,
    });

    const result = await client.GET("/statistics", {
      params: { header: AUTH_HEADER },
    });

    expect(result.response.status).toBe(400);
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(refresh).not.toHaveBeenCalled();
  });
});
