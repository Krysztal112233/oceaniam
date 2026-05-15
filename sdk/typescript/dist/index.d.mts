//#region src/types/Sqid.d.ts
type Sqid = string;
//#endregion
//#region src/types/ApplicationVO.d.ts
type ApplicationVO = {
  id: Sqid;
  comment: string | null;
  tenant_id: Sqid;
};
//#endregion
//#region src/types/Argon2Configuration.d.ts
type Argon2Configuration = {
  m_cost: number;
  t_cost: number;
  p_cost: number;
};
//#endregion
//#region src/types/PasswordConfigurationVO.d.ts
type PasswordConfigurationVO = {
  argon2: Argon2Configuration;
};
//#endregion
//#region src/types/TokenConfigurationVO.d.ts
type TokenConfigurationVO = {
  issuer: string;
  audience: Array<string>;
};
//#endregion
//#region src/types/TotpConfigurationVO.d.ts
type TotpConfigurationVO = {
  encryption_key: string;
};
//#endregion
//#region src/types/AuthConfigurationVO.d.ts
type AuthConfigurationVO = {
  token: TokenConfigurationVO;
  password: PasswordConfigurationVO;
  totp: TotpConfigurationVO;
};
//#endregion
//#region src/types/RegistrationConfigurationVO.d.ts
type RegistrationConfigurationVO = {
  enabled: boolean;
};
//#endregion
//#region src/types/ApplicationConfigurationVO.d.ts
type ApplicationConfigurationVO = {
  auth: AuthConfigurationVO;
  registration: RegistrationConfigurationVO;
};
//#endregion
//#region src/types/ApplicationDetailVO.d.ts
type ApplicationDetailVO = {
  id: Sqid;
  comment: string | null;
  tenant_id: Sqid;
  configuration: ApplicationConfigurationVO;
};
//#endregion
//#region src/types/ApplicationUserVO.d.ts
type ApplicationUserVO = {
  id: Sqid;
  email: string | null;
  phone: string | null;
  nickname: string;
};
//#endregion
//#region src/types/CreateApplicationRequest.d.ts
type CreateApplicationRequest = {
  comment: string | null;
};
//#endregion
//#region src/types/CreateApplicationResponse.d.ts
type CreateApplicationResponse = {
  tenant_id: Sqid;
  application_id: Sqid;
  comment: string | null;
};
//#endregion
//#region src/types/CreateApplicationUserRequest.d.ts
type CreateApplicationUserRequest = {
  /**
   * User email address (optional, but either phone or email must be provided)
   */
  email: string | null;
  /**
   * User phone number (optional, but either phone or email must be provided)
   */
  phone: string | null;
  /**
   * User nickname (optional, if not provided, a random name will be generated)
   */
  nickname: string | null;
  password: string;
};
//#endregion
//#region src/types/GetApplicationConfigurationResponse.d.ts
type GetApplicationConfigurationResponse = {
  configuration: ApplicationConfigurationVO;
};
//#endregion
//#region src/types/PatchApplicationRequest.d.ts
type PatchApplicationRequest = {
  comment?: string | null;
};
//#endregion
//#region src/types/PatchTokenConfigurationVO.d.ts
type PatchTokenConfigurationVO = {
  issuer: string | null;
  audience: Array<string> | null;
};
//#endregion
//#region src/types/PatchTotpConfigurationVO.d.ts
type PatchTotpConfigurationVO = {
  encryption_key: string | null;
};
//#endregion
//#region src/types/PatchAuthConfigurationVO.d.ts
type PatchAuthConfigurationVO = {
  token: PatchTokenConfigurationVO | null;
  totp: PatchTotpConfigurationVO | null;
};
//#endregion
//#region src/types/PatchRegistrationConfigurationVO.d.ts
type PatchRegistrationConfigurationVO = {
  enabled: boolean | null;
};
//#endregion
//#region src/types/PatchApplicationConfigurationRequest.d.ts
type PatchApplicationConfigurationRequest = {
  auth: PatchAuthConfigurationVO | null;
  registration: PatchRegistrationConfigurationVO | null;
};
//#endregion
//#region src/types/PatchApplicationUserCredentialsRequest.d.ts
type PatchApplicationUserCredentialsRequest = {
  password: string | null;
};
//#endregion
//#region src/types/CreateTenantRequest.d.ts
type CreateTenantRequest = {
  comment: string | null;
};
//#endregion
//#region src/pagination.d.ts
type PageInfo = {
  has_next: boolean;
  total: number;
};
type PagedResponse<T> = {
  items: T[];
  page_info: PageInfo;
};
//#endregion
//#region src/types/SignoutResponse.d.ts
type SignoutResponse = {
  msg: string;
};
//#endregion
//#region src/types/SigninChallenge.d.ts
type SigninChallenge = Record<string, never>;
//#endregion
//#region src/types/SignupResponse.d.ts
type SignupResponse = {
  jwt: string;
};
//#endregion
//#region src/types/SystemSigninResponse.d.ts
type SystemSigninResponse = SignupResponse | SigninChallenge;
//#endregion
//#region src/types/TenantVO.d.ts
type TenantVO = {
  id: Sqid;
  comment: string | null;
};
//#endregion
//#region src/types/TokenDispatchMethod.d.ts
/**
 * How an authentication token should be delivered back to the client.
 *
 * This is typically chosen by the endpoint that issues or refreshes tokens.
 *
 * The dispatch method can be controlled by the request header
 * `X-OceanIAM-Token-Dispatch`.
 *
 * - Accepted values: `cookie`, `json`, `both` (case-insensitive; whitespace is ignored)
 * - Default: missing/invalid header falls back to `both`
 *
 * Common use-cases:
 * - Browser apps often prefer `Cookie` so the token is automatically attached
 *   to subsequent requests (and can be protected with `HttpOnly`, `Secure`,
 *   `SameSite`, etc.).
 * - API clients (mobile/desktop/CLI/SDK) often prefer `Json` so they can
 *   manage the token explicitly (e.g., attach it as an `Authorization: Bearer`
 *   header).
 * - During migrations or when you need to support both kinds of clients,
 *   `Both` can be used to send the token in both places.
 */
type TokenDispatchMethod = "Cookie" | "Json" | "Both";
//#endregion
//#region src/types/SecretVO.d.ts
type SecretVO = {
  id: Sqid;
  secret: string;
  created_at: string;
  revoked_at: string | null;
  application_ids: Array<Sqid>;
};
//#endregion
//#region src/types/AdministratorVO.d.ts
type AdministratorVO = {
  id: Sqid;
  name: string;
};
//#endregion
//#region src/types/AuthVO.d.ts
type AuthVO = {
  email: string;
  password: string;
} | {
  phone: string;
  password: string;
};
//#endregion
//#region src/types/ApplicationChallengeVO.d.ts
type ApplicationChallengeVO = {
  id: string;
  application_id: Sqid;
  subject_id: string;
  factor_type: string;
  purpose: string;
  status: string;
  attempt_count: number;
  remaining_attempts: number;
  expires_at: string;
  consumed_at: string | null;
  created_at: string;
};
//#endregion
//#region src/types/ApplicationKeyVO.d.ts
type ApplicationKeyVO = {
  key_id: Sqid;
  algorithm: string;
  status: string;
  created_at: string;
  activated_at: string;
  retired_at: string;
  expires_at: string;
  revoked_at: string | null;
};
//#endregion
//#region src/types/RotateKeyResponse.d.ts
type RotateKeyResponse = {
  key: ApplicationKeyVO;
};
//#endregion
//#region src/types/CreateAdministratorRequest.d.ts
type CreateAdministratorRequest = {
  name: string;
};
//#endregion
//#region src/types/CreateAdministratorResponse.d.ts
type CreateAdministratorResponse = {
  administrator: AdministratorVO;
  initial_password: string;
};
//#endregion
//#region src/types/PatchAdministratorRequest.d.ts
type PatchAdministratorRequest = {
  name: string | null;
  password: string | null;
};
//#endregion
//#region src/types/PatchTenantRequest.d.ts
type PatchTenantRequest = {
  comment?: string | null;
};
//#endregion
//#region src/client.d.ts
type TokenGetter = () => string | null | undefined | Promise<string | null | undefined>;
type GetApplicationsQuery = {
  tenant_id: Sqid;
} & PaginationQuery;
type ApplicationUsersSortOrder = "asc" | "desc";
type PaginationQuery = {
  page: number | bigint;
  per_page: number | bigint;
} | {
  page?: undefined;
  per_page?: undefined;
};
type GetTenantsQuery = PaginationQuery;
type GetTenantUsersQuery = PaginationQuery;
type GetApplicationUsersQuery = PaginationQuery & {
  sort_order?: ApplicationUsersSortOrder;
};
type SearchApplicationUsersQuery = PaginationQuery & {
  by_nickname?: string;
  by_email?: string;
  by_id?: string;
  by_phone?: string;
};
type GetSecretsQuery = PaginationQuery;
type TenantScopedCreateApplicationRequest = Omit<CreateApplicationRequest, "tenant_id">;
declare function doSystemAuth(name: string, password: string, baseUrl?: string, dispatch?: TokenDispatchMethod): Promise<SystemSigninResponse>;
declare function doSystemSignout(jwt: string, baseUrl?: string): Promise<SignoutResponse>;
declare function doSystemRefreshToken(jwt: string, baseUrl?: string, dispatch?: TokenDispatchMethod): Promise<SystemSigninResponse>;
interface OceanIamClientConfig {
  baseUrl?: string;
  tokenGetter?: TokenGetter;
  onUnauthorized?: () => Promise<string | null | undefined>;
}
declare class OceanIamClient {
  /**
   * API paths extracted from backend `#[utoipa::path(..., path = "...")]` declarations.
   *
   * NOTE: Keep these in sync with backend routes. Do not "normalize" them unless the backend does.
   * For example, tenant-scoped application resources must stay under `/tenants/{tenant_id}/applications/...`.
   */
  private static readonly PATHS;
  private baseUrl;
  private tokenGetter?;
  private onUnauthorized?;
  constructor({
    baseUrl,
    tokenGetter,
    onUnauthorized
  }: OceanIamClientConfig);
  /**
   * Full URLs for each endpoint, built from `baseUrl` + `PATHS`.
   *
   * - If `baseUrl` is empty, returns plain paths (e.g. `/auth/tokens`), useful for same-origin setups.
   * - Otherwise, returns a concatenated URL (e.g. `https://api.example.com/auth/tokens`).
   */
  readonly endpoints: {
    readonly root: () => string;
    readonly systemSignin: () => string;
    readonly systemSignout: () => string;
    readonly systemSignup: () => string;
    readonly systemRefreshToken: () => string;
    readonly tenants: () => string;
    readonly tenant: (tenantId: string) => string;
    readonly tenantUsers: (tenantId: string) => string;
    readonly administrators: () => string;
    readonly administrator: (targetId: string) => string;
    readonly applications: (tenantId: string) => string;
    readonly application: (tenantId: string, applicationId: string) => string;
    readonly applicationConfiguration: (tenantId: string, applicationId: string) => string;
    readonly applicationJwks: (applicationId: string) => string;
    readonly applicationUsers: (tenantId: string, applicationId: string) => string;
    readonly applicationUsersCreate: (tenantId: string, applicationId: string) => string;
    readonly applicationUsersSearch: (tenantId: string, applicationId: string) => string;
    readonly applicationUserCredentials: (tenantId: string, applicationId: string, userId: string) => string;
    readonly applicationUser: (tenantId: string, applicationId: string, userId: string) => string;
    readonly applicationChallenge: (tenantId: string, applicationId: string, challengeId: string) => string;
    readonly applicationKeys: (tenantId: string, applicationId: string) => string;
    readonly applicationKey: (tenantId: string, applicationId: string, keyId: string) => string;
    readonly applicationUserSignin: (tenantId: string, applicationId: string) => string;
    readonly applicationUserSignout: (tenantId: string, applicationId: string) => string;
    readonly applicationUserRefreshToken: (tenantId: string, applicationId: string) => string;
    readonly secrets: () => string;
    readonly secret: (secretId: string) => string;
  };
  getTenants(query?: GetTenantsQuery): Promise<PagedResponse<TenantVO>>;
  getTenant(tenantId: string): Promise<TenantVO>;
  createTenant(req: CreateTenantRequest): Promise<TenantVO>;
  deleteTenant(tenantId: string): Promise<void>;
  patchTenant(tenantId: string, req: PatchTenantRequest): Promise<TenantVO>;
  getTenantUsers(tenantId: string, query?: GetTenantUsersQuery): Promise<PagedResponse<ApplicationUserVO>>;
  getApplications(query: GetApplicationsQuery): Promise<PagedResponse<ApplicationVO>>;
  getApplication(tenantId: string, applicationId: string): Promise<ApplicationDetailVO>;
  createApplication(tenantId: string, req: TenantScopedCreateApplicationRequest): Promise<CreateApplicationResponse>;
  patchApplication(tenantId: string, applicationId: string, req: PatchApplicationRequest): Promise<ApplicationDetailVO>;
  deleteApplication(tenantId: string, applicationId: string): Promise<void>;
  getApplicationUser(tenantId: string, applicationId: string, userId: string): Promise<ApplicationUserVO>;
  getApplicationUsers(tenantId: string, applicationId: string, query?: GetApplicationUsersQuery): Promise<PagedResponse<ApplicationUserVO>>;
  createApplicationUser(tenantId: string, applicationId: string, req: CreateApplicationUserRequest): Promise<ApplicationUserVO>;
  searchApplicationUsers(tenantId: string, applicationId: string, query: SearchApplicationUsersQuery): Promise<PagedResponse<ApplicationUserVO>>;
  patchApplicationUserCredentials(tenantId: string, applicationId: string, userId: string, req: PatchApplicationUserCredentialsRequest): Promise<ApplicationUserVO>;
  applicationUserSignin(tenantId: string, applicationId: string, auth: AuthVO, dispatch?: TokenDispatchMethod): Promise<SystemSigninResponse>;
  applicationUserSignout(tenantId: string, applicationId: string): Promise<SignoutResponse>;
  applicationUserRefreshToken(tenantId: string, applicationId: string, dispatch?: TokenDispatchMethod): Promise<SystemSigninResponse>;
  getApplicationConfiguration(tenantId: string, applicationId: string): Promise<GetApplicationConfigurationResponse>;
  patchApplicationConfiguration(tenantId: string, applicationId: string, req: PatchApplicationConfigurationRequest): Promise<void>;
  createSecret(): Promise<SecretVO>;
  getSecrets(query?: GetSecretsQuery): Promise<PagedResponse<SecretVO>>;
  getSecret(secretId: string): Promise<SecretVO>;
  deleteSecret(secretId: string): Promise<void>;
  getApplicationJwks(applicationId: string): Promise<unknown>;
  systemSignup(name: string, password: string): Promise<void>;
  getAdministrators(query?: PaginationQuery): Promise<PagedResponse<AdministratorVO>>;
  createAdministrator(req: CreateAdministratorRequest): Promise<CreateAdministratorResponse>;
  patchAdministrator(targetId: string, req: PatchAdministratorRequest): Promise<AdministratorVO>;
  getApplicationChallenge(tenantId: string, applicationId: string, challengeId: string): Promise<ApplicationChallengeVO>;
  getApplicationKeys(tenantId: string, applicationId: string): Promise<PagedResponse<ApplicationKeyVO>>;
  rotateApplicationKey(tenantId: string, applicationId: string): Promise<RotateKeyResponse>;
  revokeApplicationKey(tenantId: string, applicationId: string, keyId: string): Promise<void>;
  submitApplicationChallenge(tenantId: string, applicationId: string, challengeId: string, payload: unknown): Promise<SystemSigninResponse>;
  private buildUrl;
  private buildUrlWithQuery;
  private request;
  private _doRequest;
  private _doRequestWithToken;
}
//#endregion
export { type ApplicationConfigurationVO, type ApplicationKeyVO, ApplicationUsersSortOrder, type ApplicationVO, type CreateApplicationUserRequest, GetApplicationUsersQuery, GetApplicationsQuery, GetSecretsQuery, GetTenantUsersQuery, GetTenantsQuery, OceanIamClient, OceanIamClientConfig, type PageInfo, type PagedResponse, PaginationQuery, type RotateKeyResponse, SearchApplicationUsersQuery, type SecretVO, type TenantVO, TokenGetter, doSystemAuth, doSystemRefreshToken, doSystemSignout };