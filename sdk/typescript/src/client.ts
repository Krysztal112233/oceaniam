import { type ApplicationVO } from "./types/ApplicationVO";
import { type ApplicationDetailVO } from "./types/ApplicationDetailVO";
import { type ApplicationUserVO } from "./types/ApplicationUserVO";
import { type ApplicationStatisticsVO } from "./types/ApplicationStatisticsVO";
import { type ApplicationTrendsVO } from "./types/ApplicationTrendsVO";
import { type AuditLogVO } from "./types/AuditLogVO";
import { type OverviewVO } from "./types/OverviewVO";
import { type PlatformTrendsVO } from "./types/PlatformTrendsVO";
import { type AdministratorProfileVO } from "./types/AdministratorProfileVO";
import { type CreateApplicationRequest } from "./types/CreateApplicationRequest";
import { type CreateApplicationResponse } from "./types/CreateApplicationResponse";
import { type CreateApplicationUserRequest } from "./types/CreateApplicationUserRequest";
import { type EnrollTotpResponse } from "./types/EnrollTotpResponse";
import { type GetApplicationConfigurationResponse } from "./types/GetApplicationConfigurationResponse";
import { type PatchApplicationRequest } from "./types/PatchApplicationRequest";
import { type PatchApplicationConfigurationRequest } from "./types/PatchApplicationConfigurationRequest";
import { type PatchApplicationUserCredentialsRequest } from "./types/PatchApplicationUserCredentialsRequest";
import { type CreateTenantRequest } from "./types/CreateTenantRequest";
import { type PagedResponse } from "./pagination";
import { type SignoutResponse } from "./types/SignoutResponse";
import { type SystemSigninRequest } from "./types/SystemSigninRequest";
import { type SystemSigninResponse } from "./types/SystemSigninResponse";
import { type TenantVO } from "./types/TenantVO";
import { type TokenDispatchMethod } from "./types/TokenDispatchMethod";
import { type SecretVO } from "./types/SecretVO";
import { type AdministratorVO } from "./types/AdministratorVO";
import { type AuthVO } from "./types/AuthVO";
import { type ApplicationChallengeVO } from "./types/ApplicationChallengeVO";
import { type ApplicationKeyVO } from "./types/ApplicationKeyVO";
import { type CreateAdministratorRequest } from "./types/CreateAdministratorRequest";
import { type CreateAdministratorResponse } from "./types/CreateAdministratorResponse";
import { type PatchAdministratorRequest } from "./types/PatchAdministratorRequest";
import { type PatchTenantRequest } from "./types/PatchTenantRequest";

export type TokenGetter = () => string | null | undefined | Promise<string | null | undefined>;

export type GetApplicationsQuery = {
  tenant_id: string;
} & PaginationQuery;

export type ApplicationUsersSortOrder = "asc" | "desc";

export type PaginationQuery =
  | {
      page: number | bigint;
      per_page: number | bigint;
    }
  | {
      page?: undefined;
      per_page?: undefined;
    };

export type GetTenantsQuery = PaginationQuery;
export type GetTenantUsersQuery = PaginationQuery;
export type GetApplicationUsersQuery = PaginationQuery & {
  sort_order?: ApplicationUsersSortOrder;
};
export type SearchApplicationUsersQuery = PaginationQuery & {
  by_nickname?: string;
  by_email?: string;
  by_id?: string;
  by_phone?: string;
  sort_order?: ApplicationUsersSortOrder;
};
export type GetSecretsQuery = PaginationQuery;
export type AuditLogQuery = PaginationQuery & {
  audit_type?: string;
};

export type TrendQuery = {
  granularity?: "day" | "week" | "month";
  range?: number;
};

type TenantScopedCreateApplicationRequest = Omit<CreateApplicationRequest, "tenant_id">;

type QueryValue = string | number | bigint | boolean | null | undefined;

function toTokenDispatchHeader(
  method?: TokenDispatchMethod,
): "cookie" | "json" | "both" | undefined {
  if (!method) return undefined;
  if (method === "Cookie") return "cookie";
  if (method === "Json") return "json";
  if (method === "Both") return "both";
  return undefined;
}

function resolveSystemUrl(path: string, baseUrl?: string): string {
  const normalizedBase = baseUrl?.trim() ?? "";
  if (!normalizedBase) {
    if (typeof window === "undefined") {
      throw new Error(
        `Relative URL is not supported in Node.js fetch: ${path}. Provide baseUrl (e.g. http://127.0.0.1:3000).`,
      );
    }
    return path;
  }

  const base = normalizedBase.replace(/\/+$/, "");
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  return `${base}${normalizedPath}`;
}

export async function doSystemAuth(
  name: string,
  password: string,
  baseUrl?: string,
  dispatch?: TokenDispatchMethod,
): Promise<SystemSigninResponse> {
  if (typeof fetch !== "function") {
    throw new Error(
      "Global fetch is not available in this runtime. Use Node.js 18+ or provide a fetch polyfill.",
    );
  }

  const headers: Record<string, string> = {
    Accept: "application/json",
    "Content-Type": "application/json",
  };
  const tokenDispatch = toTokenDispatchHeader(dispatch);
  if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;

  const res = await fetch(resolveSystemUrl("/auth/tokens", baseUrl), {
    method: "POST",
    headers,
    body: JSON.stringify({ name, password } satisfies SystemSigninRequest),
  });

  const text = await res.text();
  if (res.ok) {
    try {
      return JSON.parse(text) as SystemSigninResponse;
    } catch {
      throw new Error("System signin succeeded but response JSON is invalid.");
    }
  }

  let msg = text || `${res.status} ${res.statusText}`;
  try {
    const json = JSON.parse(text) as unknown;
    if (
      json &&
      typeof json === "object" &&
      "msg" in json &&
      typeof (json as { msg?: unknown }).msg === "string"
    ) {
      msg = (json as { msg: string }).msg;
    }
  } catch {
    // ignore parse errors
  }

  throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
}

export async function doSystemSignout(jwt: string, baseUrl?: string): Promise<SignoutResponse> {
  if (!jwt.trim()) {
    throw new Error("JWT is required for system signout.");
  }
  if (typeof fetch !== "function") {
    throw new Error(
      "Global fetch is not available in this runtime. Use Node.js 18+ or provide a fetch polyfill.",
    );
  }

  const res = await fetch(resolveSystemUrl("/auth/tokens", baseUrl), {
    method: "DELETE",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${jwt.trim()}`,
    },
  });

  const text = await res.text();
  if (res.ok && res.status !== 203) {
    if (!text) return { msg: "" };
    try {
      return JSON.parse(text) as SignoutResponse;
    } catch {
      throw new Error("System signout succeeded but response JSON is invalid.");
    }
  }

  let msg = text || `${res.status} ${res.statusText}`;
  // NOTE: AI-generated content
  try {
    const json = JSON.parse(text) as unknown;
    if (
      json &&
      typeof json === "object" &&
      "msg" in json &&
      typeof (json as { msg?: unknown }).msg === "string"
    ) {
      msg = (json as { msg: string }).msg;
    }
  } catch {
    // ignore parse errors
  }

  throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
}

export async function doSystemRefreshToken(
  jwt: string,
  baseUrl?: string,
  dispatch?: TokenDispatchMethod,
): Promise<SystemSigninResponse> {
  if (!jwt.trim()) {
    throw new Error("JWT is required for system token refresh.");
  }
  if (typeof fetch !== "function") {
    throw new Error(
      "Global fetch is not available in this runtime. Use Node.js 18+ or provide a fetch polyfill.",
    );
  }

  const headers: Record<string, string> = {
    Accept: "application/json",
    Authorization: `Bearer ${jwt.trim()}`,
  };
  const tokenDispatch = toTokenDispatchHeader(dispatch);
  if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;

  const res = await fetch(resolveSystemUrl("/auth/tokens/refresh", baseUrl), {
    method: "POST",
    headers,
  });

  const text = await res.text();
  if (res.ok) {
    try {
      return JSON.parse(text) as SystemSigninResponse;
    } catch {
      throw new Error("System token refresh succeeded but response JSON is invalid.");
    }
  }

  let msg = text || `${res.status} ${res.statusText}`;
  try {
    const json = JSON.parse(text) as unknown;
    if (
      json &&
      typeof json === "object" &&
      "msg" in json &&
      typeof (json as { msg?: unknown }).msg === "string"
    ) {
      msg = (json as { msg: string }).msg;
    }
  } catch {
    // ignore parse errors
  }

  throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
}

export interface OceanIamClientConfig {
  baseUrl?: string;
  tokenGetter?: TokenGetter;
  onUnauthorized?: () => Promise<string | null | undefined>;
}

export class OceanIamClient {
  /**
   * API paths extracted from backend `#[utoipa::path(..., path = "...")]` declarations.
   *
   * NOTE: Keep these in sync with backend routes. Do not "normalize" them unless the backend does.
   * For example, tenant-scoped application resources must stay under `/tenants/{tenant_id}/applications/...`.
   */
  // NOTE: AI-generated content
  private static readonly PATHS = {
    root: "/",

    // SystemAuthentication (backend: endpoints/authentication.rs)
    systemAuthTokens: "/auth/tokens",
    systemAuthUsers: "/auth/users",
    systemAuthTokensRefresh: "/auth/tokens/refresh",

    // Tenants (backend: endpoints/tenants.rs)
    tenants: "/tenants",
    tenant: (tenantId: string): string => `/tenants/${encodeURIComponent(tenantId)}`,
    tenantUsers: (tenantId: string): string => `/tenants/${encodeURIComponent(tenantId)}/users`,

    // Administrators (backend: endpoints/administrators.rs)
    administrators: "/administrators",
    administrator: (targetId: string): string => `/administrators/${encodeURIComponent(targetId)}`,
    administratorSelf: "/administrators/me",

    // Applications (backend: endpoints/applications.rs)
    applications: (tenantId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications`,
    application: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}`,
    applicationConfiguration: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/configuration`,
    tenantJwks: (tenantId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/.well-known/jwks.json`,

    // ApplicationUsers
    applicationUsers: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users`,
    applicationUsersCreate: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users`,
    applicationUsersSearch: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/search`,
    applicationUserCredentials: (tenantId: string, applicationId: string, userId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}/credentials`,

    // ApplicationUser TOTP
    applicationUserTotpEnroll: (tenantId: string, applicationId: string, userId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}/totp/enroll`,
    applicationUserTotpVerify: (tenantId: string, applicationId: string, userId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}/totp/verify`,
    applicationUserTotp: (tenantId: string, applicationId: string, userId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}/totp`,

    // ApplicationUser (single user)
    applicationUser: (tenantId: string, applicationId: string, userId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}`,

    // Application Challenges (backend: endpoints/application_challenges.rs)
    applicationChallenge: (tenantId: string, applicationId: string, challengeId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/challenges/${encodeURIComponent(challengeId)}`,

    // TenantKeys (backend: endpoints/applications/keys.rs)
    tenantKeys: (tenantId: string): string => `/tenants/${encodeURIComponent(tenantId)}/keys`,
    tenantKey: (tenantId: string, keyId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/keys/${encodeURIComponent(keyId)}`,

    // Application Tokens (backend: endpoints/application_tokens.rs)
    applicationTokens: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/tokens`,
    applicationTokensRefresh: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/tokens/refresh`,

    // Secrets (backend: endpoints/secrets.rs)
    secrets: "/secrets",
    secret: (secretId: string): string => `/secrets/${encodeURIComponent(secretId)}`,
    secretBindings: (secretId: string): string =>
      `/secrets/${encodeURIComponent(secretId)}/bindings`,
    secretBinding: (secretId: string, applicationId: string): string =>
      `/secrets/${encodeURIComponent(secretId)}/bindings/${encodeURIComponent(applicationId)}`,
    applicationSecrets: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/secrets`,

    // Statistics (backend: endpoints/statistics.rs)
    statistics: "/statistics",
    statisticsTrends: "/statistics/trends",
    applicationStatistics: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/statistics`,
    applicationStatisticsTrends: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/statistics/trends`,
    applicationAudits: (tenantId: string, applicationId: string): string =>
      `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/audits`,

    // Audits (backend: endpoints/audits.rs)
    audits: "/audits",
  } as const;

  private baseUrl: string;
  private tokenGetter?: TokenGetter;
  private onUnauthorized?: () => Promise<string | null | undefined>;

  constructor({ baseUrl, tokenGetter, onUnauthorized }: OceanIamClientConfig) {
    this.baseUrl = baseUrl?.trim() ?? "";
    this.tokenGetter = tokenGetter;
    this.onUnauthorized = onUnauthorized;
  }

  /**
   * Full URLs for each endpoint, built from `baseUrl` + `PATHS`.
   *
   * - If `baseUrl` is empty, returns plain paths (e.g. `/auth/tokens`), useful for same-origin setups.
   * - Otherwise, returns a concatenated URL (e.g. `https://api.example.com/auth/tokens`).
   */
  public readonly endpoints = {
    root: (): string => this.buildUrl(OceanIamClient.PATHS.root),

    // SystemAuthentication
    systemSignin: (): string => this.buildUrl(OceanIamClient.PATHS.systemAuthTokens),
    systemSignout: (): string => this.buildUrl(OceanIamClient.PATHS.systemAuthTokens),
    systemSignup: (): string => this.buildUrl(OceanIamClient.PATHS.systemAuthUsers),
    systemRefreshToken: (): string => this.buildUrl(OceanIamClient.PATHS.systemAuthTokensRefresh),

    // Tenants
    tenants: (): string => this.buildUrl(OceanIamClient.PATHS.tenants),
    tenant: (tenantId: string): string => this.buildUrl(OceanIamClient.PATHS.tenant(tenantId)),
    tenantUsers: (tenantId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.tenantUsers(tenantId)),

    // Administrators
    administrators: (): string => this.buildUrl(OceanIamClient.PATHS.administrators),
    administrator: (targetId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.administrator(targetId)),
    administratorSelf: (): string => this.buildUrl(OceanIamClient.PATHS.administratorSelf),

    // Applications
    applications: (tenantId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applications(tenantId)),
    application: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.application(tenantId, applicationId)),
    applicationConfiguration: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationConfiguration(tenantId, applicationId)),
    applicationJwks: (tenantId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.tenantJwks(tenantId)),

    // ApplicationUsers
    applicationUsers: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationUsers(tenantId, applicationId)),
    applicationUsersCreate: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationUsersCreate(tenantId, applicationId)),
    applicationUsersSearch: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationUsersSearch(tenantId, applicationId)),
    applicationUserCredentials: (tenantId: string, applicationId: string, userId: string): string =>
      this.buildUrl(
        OceanIamClient.PATHS.applicationUserCredentials(tenantId, applicationId, userId),
      ),

    // ApplicationUser TOTP
    applicationUserTotpEnroll: (tenantId: string, applicationId: string, userId: string): string =>
      this.buildUrl(
        OceanIamClient.PATHS.applicationUserTotpEnroll(tenantId, applicationId, userId),
      ),
    applicationUserTotpVerify: (tenantId: string, applicationId: string, userId: string): string =>
      this.buildUrl(
        OceanIamClient.PATHS.applicationUserTotpVerify(tenantId, applicationId, userId),
      ),
    applicationUserTotp: (tenantId: string, applicationId: string, userId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationUserTotp(tenantId, applicationId, userId)),

    // ApplicationUser (single user)
    applicationUser: (tenantId: string, applicationId: string, userId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationUser(tenantId, applicationId, userId)),

    // Application Challenges
    applicationChallenge: (tenantId: string, applicationId: string, challengeId: string): string =>
      this.buildUrl(
        OceanIamClient.PATHS.applicationChallenge(tenantId, applicationId, challengeId),
      ),

    // Tenant Keys
    tenantKeys: (tenantId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.tenantKeys(tenantId)),
    tenantKey: (tenantId: string, keyId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.tenantKey(tenantId, keyId)),

    // Application Tokens
    applicationUserSignin: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationTokens(tenantId, applicationId)),
    applicationUserSignout: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationTokens(tenantId, applicationId)),
    applicationUserRefreshToken: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationTokensRefresh(tenantId, applicationId)),

    // Secrets
    secrets: (): string => this.buildUrl(OceanIamClient.PATHS.secrets),
    secret: (secretId: string): string => this.buildUrl(OceanIamClient.PATHS.secret(secretId)),
    secretBindings: (secretId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.secretBindings(secretId)),
    secretBinding: (secretId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.secretBinding(secretId, applicationId)),

    // Statistics
    statistics: (): string => this.buildUrl(OceanIamClient.PATHS.statistics),
    statisticsTrends: (): string => this.buildUrl(OceanIamClient.PATHS.statisticsTrends),
    applicationStatistics: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationStatistics(tenantId, applicationId)),
    applicationStatisticsTrends: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationStatisticsTrends(tenantId, applicationId)),
    applicationAudits: (tenantId: string, applicationId: string): string =>
      this.buildUrl(OceanIamClient.PATHS.applicationAudits(tenantId, applicationId)),

    // Audits
    audits: (): string => this.buildUrl(OceanIamClient.PATHS.audits),
  } as const;

  public async getTenants(query?: GetTenantsQuery): Promise<PagedResponse<TenantVO>> {
    const { page, per_page } = query ?? {};
    return this.request<PagedResponse<TenantVO>>({
      method: "GET",
      url: this.endpoints.tenants(),
      query: { page, per_page },
    });
  }

  public async getTenant(tenantId: string): Promise<TenantVO> {
    return this.request<TenantVO>({
      method: "GET",
      url: this.endpoints.tenant(tenantId),
    });
  }

  public async createTenant(req: CreateTenantRequest): Promise<TenantVO> {
    return this.request<TenantVO>({
      method: "POST",
      url: this.endpoints.tenants(),
      body: req,
    });
  }

  public async deleteTenant(tenantId: string): Promise<void> {
    await this.request<unknown>({
      method: "DELETE",
      url: this.endpoints.tenant(tenantId),
    });
  }

  public async patchTenant(tenantId: string, req: PatchTenantRequest): Promise<TenantVO> {
    return this.request<TenantVO>({
      method: "PATCH",
      url: this.endpoints.tenant(tenantId),
      body: req,
    });
  }

  public async getTenantUsers(
    tenantId: string,
    query?: GetTenantUsersQuery,
  ): Promise<PagedResponse<ApplicationUserVO>> {
    const { page, per_page } = query ?? {};
    return this.request<PagedResponse<ApplicationUserVO>>({
      method: "GET",
      url: this.endpoints.tenantUsers(tenantId),
      query: { page, per_page },
    });
  }

  public async getApplications(query: GetApplicationsQuery): Promise<PagedResponse<ApplicationVO>> {
    const { tenant_id, page, per_page } = query;
    return this.request<PagedResponse<ApplicationVO>>({
      method: "GET",
      url: this.endpoints.applications(tenant_id),
      query: { page, per_page },
    });
  }

  public async getApplication(
    tenantId: string,
    applicationId: string,
  ): Promise<ApplicationDetailVO> {
    return this.request<ApplicationDetailVO>({
      method: "GET",
      url: this.endpoints.application(tenantId, applicationId),
    });
  }

  public async createApplication(
    tenantId: string,
    req: TenantScopedCreateApplicationRequest,
  ): Promise<CreateApplicationResponse> {
    return this.request<CreateApplicationResponse>({
      method: "POST",
      url: this.endpoints.applications(tenantId),
      body: req,
    });
  }

  public async patchApplication(
    tenantId: string,
    applicationId: string,
    req: PatchApplicationRequest,
  ): Promise<ApplicationDetailVO> {
    return this.request<ApplicationDetailVO>({
      method: "PATCH",
      url: this.endpoints.application(tenantId, applicationId),
      body: req,
    });
  }

  public async deleteApplication(tenantId: string, applicationId: string): Promise<void> {
    await this.request<unknown>({
      method: "DELETE",
      url: this.endpoints.application(tenantId, applicationId),
    });
  }

  public async getApplicationUser(
    tenantId: string,
    applicationId: string,
    userId: string,
  ): Promise<ApplicationUserVO> {
    return this.request<ApplicationUserVO>({
      method: "GET",
      url: this.endpoints.applicationUser(tenantId, applicationId, userId),
    });
  }

  public async getApplicationUsers(
    tenantId: string,
    applicationId: string,
    query?: GetApplicationUsersQuery,
  ): Promise<PagedResponse<ApplicationUserVO>> {
    const { page, per_page, sort_order } = query ?? {};
    return this.request<PagedResponse<ApplicationUserVO>>({
      method: "GET",
      url: this.endpoints.applicationUsers(tenantId, applicationId),
      query: { page, per_page, sort_order },
    });
  }

  public async createApplicationUser(
    tenantId: string,
    applicationId: string,
    req: CreateApplicationUserRequest,
  ): Promise<ApplicationUserVO> {
    return this.request<ApplicationUserVO>({
      method: "POST",
      url: this.endpoints.applicationUsersCreate(tenantId, applicationId),
      body: req,
    });
  }

  public async searchApplicationUsers(
    tenantId: string,
    applicationId: string,
    query: SearchApplicationUsersQuery,
  ): Promise<PagedResponse<ApplicationUserVO>> {
    const { by_nickname, by_email, by_id, by_phone, page, per_page, sort_order } = query;
    return this.request<PagedResponse<ApplicationUserVO>>({
      method: "GET",
      url: this.endpoints.applicationUsersSearch(tenantId, applicationId),
      query: {
        by_nickname,
        by_email,
        by_id,
        by_phone,
        page,
        per_page,
        sort_order,
      },
    });
  }

  public async patchApplicationUserCredentials(
    tenantId: string,
    applicationId: string,
    userId: string,
    req: PatchApplicationUserCredentialsRequest,
  ): Promise<ApplicationUserVO> {
    return this.request<ApplicationUserVO>({
      method: "PATCH",
      url: this.endpoints.applicationUserCredentials(tenantId, applicationId, userId),
      body: req,
    });
  }

  public async applicationUserSignin(
    tenantId: string,
    applicationId: string,
    auth: AuthVO,
    dispatch?: TokenDispatchMethod,
  ): Promise<SystemSigninResponse> {
    const headers: Record<string, string> = {};
    const tokenDispatch = toTokenDispatchHeader(dispatch);
    if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;

    return this.request<SystemSigninResponse>({
      method: "POST",
      url: this.endpoints.applicationUserSignin(tenantId, applicationId),
      body: auth,
      headers,
    });
  }

  public async applicationUserSignout(
    tenantId: string,
    applicationId: string,
  ): Promise<SignoutResponse> {
    return this.request<SignoutResponse>({
      method: "DELETE",
      url: this.endpoints.applicationUserSignout(tenantId, applicationId),
    });
  }

  public async applicationUserRefreshToken(
    tenantId: string,
    applicationId: string,
    dispatch?: TokenDispatchMethod,
  ): Promise<SystemSigninResponse> {
    const headers: Record<string, string> = {};
    const tokenDispatch = toTokenDispatchHeader(dispatch);
    if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;

    return this.request<SystemSigninResponse>({
      method: "POST",
      url: this.endpoints.applicationUserRefreshToken(tenantId, applicationId),
      headers,
    });
  }

  public async getApplicationConfiguration(
    tenantId: string,
    applicationId: string,
  ): Promise<GetApplicationConfigurationResponse> {
    return this.request<GetApplicationConfigurationResponse>({
      method: "GET",
      url: this.endpoints.applicationConfiguration(tenantId, applicationId),
    });
  }

  public async patchApplicationConfiguration(
    tenantId: string,
    applicationId: string,
    req: PatchApplicationConfigurationRequest,
  ): Promise<void> {
    await this.request<unknown>({
      method: "PATCH",
      url: this.endpoints.applicationConfiguration(tenantId, applicationId),
      body: req,
    });
  }

  public async createSecret(): Promise<SecretVO> {
    return this.request<SecretVO>({
      method: "POST",
      url: this.endpoints.secrets(),
    });
  }

  public async getSecrets(query?: GetSecretsQuery): Promise<PagedResponse<SecretVO>> {
    const { page, per_page } = query ?? {};
    return this.request<PagedResponse<SecretVO>>({
      method: "GET",
      url: this.endpoints.secrets(),
      query: { page, per_page },
    });
  }

  public async getSecret(secretId: string): Promise<SecretVO> {
    return this.request<SecretVO>({
      method: "GET",
      url: this.endpoints.secret(secretId),
    });
  }

  public async deleteSecret(secretId: string): Promise<void> {
    await this.request<unknown>({
      method: "DELETE",
      url: this.endpoints.secret(secretId),
    });
  }

  public async bindSecretToApplication(secretId: string, applicationId: string): Promise<void> {
    await this.request<unknown>({
      method: "POST",
      url: this.endpoints.secretBindings(secretId),
      body: { application_id: applicationId },
    });
  }

  public async unbindSecretFromApplication(secretId: string, applicationId: string): Promise<void> {
    await this.request<unknown>({
      method: "DELETE",
      url: this.endpoints.secretBinding(secretId, applicationId),
    });
  }

  public async getApplicationSecrets(
    tenantId: string,
    applicationId: string,
  ): Promise<PagedResponse<SecretVO>> {
    return this.request<PagedResponse<SecretVO>>({
      method: "GET",
      url: this.endpoints.applicationSecrets(tenantId, applicationId),
    });
  }

  public async getStatistics(): Promise<OverviewVO> {
    return this.request<OverviewVO>({
      method: "GET",
      url: this.endpoints.statistics(),
    });
  }

  public async getStatisticsTrends(query?: TrendQuery): Promise<PlatformTrendsVO> {
    const { granularity, range } = query ?? {};
    return this.request<PlatformTrendsVO>({
      method: "GET",
      url: this.endpoints.statisticsTrends(),
      query: { granularity, range },
    });
  }

  public async getApplicationStatistics(
    tenantId: string,
    applicationId: string,
  ): Promise<ApplicationStatisticsVO> {
    return this.request<ApplicationStatisticsVO>({
      method: "GET",
      url: this.endpoints.applicationStatistics(tenantId, applicationId),
    });
  }

  public async getApplicationStatisticsTrends(
    tenantId: string,
    applicationId: string,
    query?: TrendQuery,
  ): Promise<ApplicationTrendsVO> {
    const { granularity, range } = query ?? {};
    return this.request<ApplicationTrendsVO>({
      method: "GET",
      url: this.endpoints.applicationStatisticsTrends(tenantId, applicationId),
      query: { granularity, range },
    });
  }

  public async getAudits(query?: AuditLogQuery): Promise<PagedResponse<AuditLogVO>> {
    const { page, per_page, audit_type } = query ?? {};
    return this.request<PagedResponse<AuditLogVO>>({
      method: "GET",
      url: this.endpoints.audits(),
      query: { page, per_page, audit_type },
    });
  }

  public async getApplicationAudits(
    tenantId: string,
    applicationId: string,
    query?: AuditLogQuery,
  ): Promise<PagedResponse<AuditLogVO>> {
    const { page, per_page, audit_type } = query ?? {};
    return this.request<PagedResponse<AuditLogVO>>({
      method: "GET",
      url: this.endpoints.applicationAudits(tenantId, applicationId),
      query: { page, per_page, audit_type },
    });
  }

  public async getTenantJwks(tenantId: string): Promise<unknown> {
    return this.request<unknown>({
      method: "GET",
      url: this.endpoints.applicationJwks(tenantId),
      auth: "none",
    });
  }

  public async systemSignup(name: string, password: string): Promise<void> {
    await this.request<unknown>({
      method: "POST",
      url: this.endpoints.systemSignup(),
      body: { name, password } satisfies SystemSigninRequest,
    });
  }

  public async getAdministrators(query?: PaginationQuery): Promise<PagedResponse<AdministratorVO>> {
    const { page, per_page } = query ?? {};
    return this.request<PagedResponse<AdministratorVO>>({
      method: "GET",
      url: this.endpoints.administrators(),
      query: { page, per_page },
    });
  }

  public async createAdministrator(
    req: CreateAdministratorRequest,
  ): Promise<CreateAdministratorResponse> {
    return this.request<CreateAdministratorResponse>({
      method: "POST",
      url: this.endpoints.administrators(),
      body: req,
    });
  }

  public async patchAdministrator(
    targetId: string,
    req: PatchAdministratorRequest,
  ): Promise<AdministratorVO> {
    return this.request<AdministratorVO>({
      method: "PATCH",
      url: this.endpoints.administrator(targetId),
      body: req,
    });
  }

  public async getAdministratorSelf(): Promise<AdministratorProfileVO> {
    return this.request<AdministratorProfileVO>({
      method: "GET",
      url: this.endpoints.administratorSelf(),
    });
  }

  public async getApplicationChallenge(
    tenantId: string,
    applicationId: string,
    challengeId: string,
  ): Promise<ApplicationChallengeVO> {
    return this.request<ApplicationChallengeVO>({
      method: "GET",
      url: this.endpoints.applicationChallenge(tenantId, applicationId, challengeId),
    });
  }

  public async getTenantKeys(tenantId: string): Promise<PagedResponse<ApplicationKeyVO>> {
    return this.request<PagedResponse<ApplicationKeyVO>>({
      method: "GET",
      url: this.endpoints.tenantKeys(tenantId),
    });
  }

  public async rotateTenantKey(tenantId: string): Promise<void> {
    await this.request<unknown>({
      method: "POST",
      url: this.endpoints.tenantKeys(tenantId),
    });
  }

  public async revokeTenantKey(tenantId: string, keyId: string): Promise<void> {
    await this.request<unknown>({
      method: "DELETE",
      url: this.endpoints.tenantKey(tenantId, keyId),
    });
  }

  public async enrollTotp(
    tenantId: string,
    applicationId: string,
    userId: string,
  ): Promise<EnrollTotpResponse> {
    return this.request<EnrollTotpResponse>({
      method: "POST",
      url: this.endpoints.applicationUserTotpEnroll(tenantId, applicationId, userId),
    });
  }

  public async verifyTotpEnrollment(
    tenantId: string,
    applicationId: string,
    userId: string,
    code: string,
  ): Promise<void> {
    await this.request<unknown>({
      method: "POST",
      url: this.endpoints.applicationUserTotpVerify(tenantId, applicationId, userId),
      body: { code },
    });
  }

  public async removeTotp(tenantId: string, applicationId: string, userId: string): Promise<void> {
    await this.request<unknown>({
      method: "DELETE",
      url: this.endpoints.applicationUserTotp(tenantId, applicationId, userId),
    });
  }

  public async submitApplicationChallenge(
    tenantId: string,
    applicationId: string,
    challengeId: string,
    payload: unknown,
  ): Promise<SystemSigninResponse> {
    return this.request<SystemSigninResponse>({
      method: "POST",
      url: this.endpoints.applicationChallenge(tenantId, applicationId, challengeId),
      body: payload as object,
    });
  }

  public async getRoot(): Promise<void> {
    await this.request<unknown>({
      method: "GET",
      url: this.endpoints.root(),
      auth: "none",
    });
  }

  private buildUrl(path: string): string {
    if (!this.baseUrl) return path;

    const base = this.baseUrl.replace(/\/+$/, "");
    const normalizedPath = path.startsWith("/") ? path : `/${path}`;
    return `${base}${normalizedPath}`;
  }

  private buildUrlWithQuery(url: string, query?: Record<string, QueryValue>): string {
    if (!query) return url;

    const params = new URLSearchParams();
    for (const [key, value] of Object.entries(query)) {
      if (value === undefined || value === null) continue;
      params.set(key, typeof value === "bigint" ? value.toString() : String(value));
    }

    const qs = params.toString();
    if (!qs) return url;
    return url.includes("?") ? `${url}&${qs}` : `${url}?${qs}`;
  }

  private async request<T>(opts: {
    method: "GET" | "POST" | "PATCH" | "DELETE";
    url: string;
    query?: Record<string, QueryValue>;
    body?: object;
    auth?: "required" | "none";
    headers?: Record<string, string>;
  }): Promise<T> {
    const result = await this._doRequest<T>(opts);

    if (result.status === "unauthorized" && this.onUnauthorized && opts.auth !== "none") {
      const newToken = await this.onUnauthorized();
      if (newToken?.trim()) {
        return this._doRequestWithToken<T>(opts, newToken.trim());
      }
    }

    if (result.status === "unauthorized" || result.status === "error") {
      throw result.error!;
    }

    return result.value as T;
  }

  private async _doRequest<T>(opts: {
    method: "GET" | "POST" | "PATCH" | "DELETE";
    url: string;
    query?: Record<string, QueryValue>;
    body?: object;
    auth?: "required" | "none";
    headers?: Record<string, string>;
  }): Promise<
    | { status: "ok"; value: T; error?: undefined }
    | { status: "unauthorized"; value?: undefined; error: Error }
    | { status: "error"; value?: undefined; error: Error }
  > {
    const { method, url, query, body, auth = "required", headers: extraHeaders } = opts;

    const finalUrl = this.buildUrlWithQuery(url, query);

    const headers: Record<string, string> = {
      Accept: "application/json",
      ...extraHeaders,
    };

    if (auth !== "none") {
      const token = (await this.tokenGetter?.())?.trim();
      if (!token) {
        return {
          status: "error",
          error: new Error(
            "Missing auth token. Provide OceanIamClientConfig.tokenGetter that returns a non-empty token string.",
          ),
        };
      }
      headers.Authorization = `Bearer ${token}`;
    }

    const hasBody = body !== undefined;
    if (hasBody) {
      headers["Content-Type"] = "application/json";
    }

    const res = await fetch(finalUrl, {
      method,
      headers,
      body: hasBody ? JSON.stringify(body) : undefined,
    });

    if (res.status === 401 || res.status === 203) {
      return {
        status: "unauthorized",
        error: new Error(`HTTP ${res.status}`),
      };
    }

    if (res.ok) {
      const text = await res.text();
      if (!text) return { status: "ok", value: undefined as T };

      try {
        return { status: "ok", value: JSON.parse(text) as T };
      } catch (e) {
        return {
          status: "error",
          error: new Error(
            `Failed to parse JSON response from ${method} ${finalUrl}: ${String(e)}`,
          ),
        };
      }
    }

    const errorText = await res.text();
    const msg = (() => {
      try {
        const json = JSON.parse(errorText) as unknown;
        if (
          json &&
          typeof json === "object" &&
          "msg" in json &&
          typeof (json as { msg?: unknown }).msg === "string"
        ) {
          return (json as { msg: string }).msg;
        }
      } catch {
        // ignore JSON parse errors
      }
      return errorText || `${res.status} ${res.statusText}`;
    })();

    return {
      status: "error",
      error: new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`),
    };
  }

  private async _doRequestWithToken<T>(
    opts: {
      method: "GET" | "POST" | "PATCH" | "DELETE";
      url: string;
      query?: Record<string, QueryValue>;
      body?: object;
      auth?: "required" | "none";
      headers?: Record<string, string>;
    },
    token: string,
  ): Promise<T> {
    const { method, url, query, body, headers: extraHeaders } = opts;

    const finalUrl = this.buildUrlWithQuery(url, query);

    const headers: Record<string, string> = {
      Accept: "application/json",
      Authorization: `Bearer ${token}`,
      ...extraHeaders,
    };

    const hasBody = body !== undefined;
    if (hasBody) {
      headers["Content-Type"] = "application/json";
    }

    const res = await fetch(finalUrl, {
      method,
      headers,
      body: hasBody ? JSON.stringify(body) : undefined,
    });

    if (res.ok && res.status !== 203) {
      const text = await res.text();
      if (!text) return undefined as T;

      try {
        return JSON.parse(text) as T;
      } catch (e) {
        throw new Error(`Failed to parse JSON response from ${method} ${finalUrl}: ${String(e)}`);
      }
    }

    const errorText = await res.text();
    const msg = (() => {
      try {
        const json = JSON.parse(errorText) as unknown;
        if (
          json &&
          typeof json === "object" &&
          "msg" in json &&
          typeof (json as { msg?: unknown }).msg === "string"
        ) {
          return (json as { msg: string }).msg;
        }
      } catch {
        // ignore JSON parse errors
      }
      return errorText || `${res.status} ${res.statusText}`;
    })();

    throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
  }
}
