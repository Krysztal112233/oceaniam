import type { components } from "@oceaniam/sdk";

export type ApplicationConfiguration =
  components["schemas"]["ApplicationConfigurationVO"];
export type PatchApplicationConfiguration =
  components["schemas"]["PatchApplicationConfigurationRequest"];

export const MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS = 2_147_483_647;

export interface ApplicationConfigurationForm {
  issuer: string;
  audience: string;
  registrationEnabled: boolean;
  developmentAccountsEnabled: boolean;
  defaultTtlSeconds: string | number;
  maxTtlSeconds: string | number;
  allowInsecureLoopbackRedirectUris: boolean;
}

export type ConfigurationError =
  "issuerRequired" | "defaultTtlInvalid" | "maxTtlInvalid" | "ttlOrder";

export function applicationConfigurationToForm(
  configuration: ApplicationConfiguration,
): ApplicationConfigurationForm {
  const developmentAccounts = configuration.development_accounts;
  return {
    issuer: configuration.auth.token.issuer,
    audience: configuration.auth.token.audience.join("\n"),
    registrationEnabled: configuration.registration.enabled,
    developmentAccountsEnabled: developmentAccounts?.enabled ?? true,
    defaultTtlSeconds: String(developmentAccounts?.default_ttl_seconds ?? 3600),
    maxTtlSeconds: String(developmentAccounts?.max_ttl_seconds ?? 86400),
    allowInsecureLoopbackRedirectUris:
      configuration.oidc?.allow_insecure_loopback_redirect_uris ?? false,
  };
}

function parseTtl(value: string | number): number | null {
  const normalized = String(value).trim();
  if (!/^[1-9]\d*$/.test(normalized)) return null;
  const parsed = Number(normalized);
  return Number.isSafeInteger(parsed) &&
    parsed <= MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS
    ? parsed
    : null;
}

export function validateApplicationConfiguration(
  form: ApplicationConfigurationForm,
): ConfigurationError[] {
  const errors: ConfigurationError[] = [];
  if (!form.issuer.trim()) errors.push("issuerRequired");

  const defaultTtl = parseTtl(form.defaultTtlSeconds);
  const maxTtl = parseTtl(form.maxTtlSeconds);
  if (defaultTtl === null) errors.push("defaultTtlInvalid");
  if (maxTtl === null) errors.push("maxTtlInvalid");
  if (defaultTtl !== null && maxTtl !== null && defaultTtl > maxTtl)
    errors.push("ttlOrder");

  return errors;
}

export function applicationConfigurationPatch(
  form: ApplicationConfigurationForm,
): PatchApplicationConfiguration {
  if (validateApplicationConfiguration(form).length > 0)
    throw new RangeError("Invalid application configuration");

  return {
    auth: {
      token: {
        issuer: form.issuer.trim(),
        audience: form.audience
          .split(/[\n,]/)
          .map((value) => value.trim())
          .filter(Boolean),
      },
    },
    registration: { enabled: form.registrationEnabled },
    development_accounts: {
      enabled: form.developmentAccountsEnabled,
      default_ttl_seconds: Number(form.defaultTtlSeconds),
      max_ttl_seconds: Number(form.maxTtlSeconds),
    },
    oidc: {
      allow_insecure_loopback_redirect_uris:
        form.allowInsecureLoopbackRedirectUris,
    },
  };
}
