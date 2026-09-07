import type { components } from "@oceaniam/sdk";

export type AccountType = "permanent" | "development";
export type TtlPreset = "3600" | "86400" | "604800" | "custom";
export type DevelopmentOptions = components["schemas"]["DevAccountOptions"];

export function developmentOptions(
  accountType: AccountType,
  preset: TtlPreset,
  customTtl: number,
): DevelopmentOptions | undefined {
  if (accountType === "permanent") return undefined;
  const ttl = preset === "custom" ? customTtl : Number(preset);
  if (!Number.isInteger(ttl) || ttl < 1 || ttl > 2_147_483_647) {
    throw new RangeError("TTL must be an integer between 1 and 2147483647.");
  }
  return { ttl_seconds: ttl };
}
