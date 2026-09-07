import { describe, expect, it } from "vitest";

import { developmentOptions } from "@/features/users/development";

describe("developmentOptions", () => {
  it("omits development settings for a permanent account", () => {
    expect(developmentOptions("permanent", "3600", 3600)).toBeUndefined();
  });

  it("maps presets and custom TTL values to the backend contract", () => {
    expect(developmentOptions("development", "3600", 1)).toEqual({
      ttl_seconds: 3600,
    });
    expect(developmentOptions("development", "custom", 42)).toEqual({
      ttl_seconds: 42,
    });
  });

  it("rejects values outside the pgmq delay range", () => {
    expect(() => developmentOptions("development", "custom", 0)).toThrow(
      RangeError,
    );
    expect(() =>
      developmentOptions("development", "custom", 2_147_483_648),
    ).toThrow(RangeError);
    expect(() => developmentOptions("development", "custom", 1.5)).toThrow(
      RangeError,
    );
  });
});
