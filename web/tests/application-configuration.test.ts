// NOTE: AI-generated test
import { describe, expect, it } from "vitest";

import {
  applicationConfigurationPatch,
  applicationConfigurationToForm,
  MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS,
  validateApplicationConfiguration,
  type ApplicationConfiguration,
  type ApplicationConfigurationForm,
} from "@/features/applications/configuration";

const configuration: ApplicationConfiguration = {
  auth: {
    token: { issuer: "OceanIAM", audience: ["api", "admin"] },
    password: { argon2: { m_cost: 12288, t_cost: 3, p_cost: 1 } },
  },
  registration: { enabled: false },
};

function validForm(): ApplicationConfigurationForm {
  return {
    issuer: "OceanIAM",
    audience: "api\n admin, ,",
    registrationEnabled: true,
    developmentAccountsEnabled: true,
    defaultTtlSeconds: "3600",
    maxTtlSeconds: "86400",
    allowInsecureLoopbackRedirectUris: false,
  };
}

describe("application configuration form", () => {
  it("uses backward-compatible development and OIDC defaults", () => {
    expect(applicationConfigurationToForm(configuration)).toEqual({
      issuer: "OceanIAM",
      audience: "api\nadmin",
      registrationEnabled: false,
      developmentAccountsEnabled: true,
      defaultTtlSeconds: "3600",
      maxTtlSeconds: "86400",
      allowInsecureLoopbackRedirectUris: false,
    });
  });

  it("maps every editable field to the generated patch contract", () => {
    expect(
      applicationConfigurationPatch({
        ...validForm(),
        allowInsecureLoopbackRedirectUris: true,
      }),
    ).toEqual({
      auth: { token: { issuer: "OceanIAM", audience: ["api", "admin"] } },
      registration: { enabled: true },
      development_accounts: {
        enabled: true,
        default_ttl_seconds: 3600,
        max_ttl_seconds: 86400,
      },
      oidc: { allow_insecure_loopback_redirect_uris: true },
    });
  });

  it("rejects empty issuers and invalid TTL boundaries", () => {
    expect(
      validateApplicationConfiguration({ ...validForm(), issuer: " " }),
    ).toContain("issuerRequired");
    for (const value of ["0", "1.5", "abc", "2147483648"]) {
      expect(
        validateApplicationConfiguration({
          ...validForm(),
          defaultTtlSeconds: value,
        }),
      ).toContain("defaultTtlInvalid");
    }
    expect(
      validateApplicationConfiguration({
        ...validForm(),
        defaultTtlSeconds: "10",
        maxTtlSeconds: "9",
      }),
    ).toContain("ttlOrder");
  });

  it("accepts numeric values emitted by number inputs", () => {
    const form = {
      ...validForm(),
      defaultTtlSeconds: 3600,
      maxTtlSeconds: 86400,
    };
    expect(validateApplicationConfiguration(form)).toEqual([]);
    expect(
      applicationConfigurationPatch(form).development_accounts,
    ).toMatchObject({ default_ttl_seconds: 3600, max_ttl_seconds: 86400 });
  });

  it("accepts the maximum supported TTL", () => {
    const form = {
      ...validForm(),
      defaultTtlSeconds: String(MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS),
      maxTtlSeconds: String(MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS),
    };
    expect(validateApplicationConfiguration(form)).toEqual([]);
    expect(
      applicationConfigurationPatch(form).development_accounts,
    ).toMatchObject({
      default_ttl_seconds: MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS,
      max_ttl_seconds: MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS,
    });
  });
});
