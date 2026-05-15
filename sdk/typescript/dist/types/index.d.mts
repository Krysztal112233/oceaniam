//#region src/types/Sqid.d.ts
type Sqid = string;
//#endregion
//#region src/types/AdministratorVO.d.ts
type AdministratorVO = {
  id: Sqid;
  name: string;
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
//#region src/types/ApplicationUsersSortOrder.d.ts
type ApplicationUsersSortOrder = "asc" | "desc";
//#endregion
//#region src/types/ApplicationUserVO.d.ts
type ApplicationUserVO = {
  id: Sqid;
  email: string | null;
  phone: string | null;
  nickname: string;
};
//#endregion
//#region src/types/ApplicationVO.d.ts
type ApplicationVO = {
  id: Sqid;
  comment: string | null;
  tenant_id: Sqid;
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
//#region src/types/Claim.d.ts
type Claim = {
  /**
   * Subject
   *
   * The subject of the token, typically the user's unique identifier (e.g., UUID)
   */
  sub: string;
  /**
   * Expiration Time
   *
   * Token expiration time (Unix timestamp, seconds)
   */
  exp: bigint;
  /**
   * Issued At
   *
   * Token issuance time (Unix timestamp, seconds)
   */
  iat: bigint;
  /**
   * Issuer
   *
   * Token issuer (optional), e.g., "oceaniam-auth"
   */
  iss: string | null;
  /**
   * Audience
   *
   * Token audience (optional), represents the intended recipient of the token
   */
  aud: Array<string> | null;
  /**
   * JWT ID
   *
   * Unique identifier for the token, used to prevent replay attacks
   */
  jti: string;
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
//#region src/types/CreateTenantRequest.d.ts
type CreateTenantRequest = {
  comment: string | null;
};
//#endregion
//#region src/types/ErrorResponse.d.ts
type ErrorResponse = {
  msg: string;
  error_id: string;
};
//#endregion
//#region src/types/GetApplicationConfigurationResponse.d.ts
type GetApplicationConfigurationResponse = {
  configuration: ApplicationConfigurationVO;
};
//#endregion
//#region src/types/GetTenantsRequest.d.ts
type GetTenantsRequest = {
  page: bigint;
  per_page: bigint;
};
//#endregion
//#region src/types/PatchAdministratorRequest.d.ts
type PatchAdministratorRequest = {
  name: string | null;
  password: string | null;
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
//#region src/types/PatchApplicationRequest.d.ts
type PatchApplicationRequest = {
  comment?: string | null;
};
//#endregion
//#region src/types/PatchApplicationUserCredentialsRequest.d.ts
type PatchApplicationUserCredentialsRequest = {
  password: string | null;
};
//#endregion
//#region src/types/PatchTenantRequest.d.ts
type PatchTenantRequest = {
  comment?: string | null;
};
//#endregion
//#region src/types/RotateKeyResponse.d.ts
type RotateKeyResponse = {
  key: ApplicationKeyVO;
};
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
//#region src/types/SigninChallenge.d.ts
type SigninChallenge = Record<string, never>;
//#endregion
//#region src/types/SigninRequest.d.ts
type SigninRequest = {
  auth: AuthVO;
};
//#endregion
//#region src/types/SigninResponse.d.ts
type SigninResponse = {
  jwt: string;
};
//#endregion
//#region src/types/SignupResponse.d.ts
type SignupResponse = {
  jwt: string;
};
//#endregion
//#region src/types/SigninResponseSchema.d.ts
type SigninResponseSchema = SignupResponse | SigninChallenge;
//#endregion
//#region src/types/SignoutResponse.d.ts
type SignoutResponse = {
  msg: string;
};
//#endregion
//#region src/types/SystemClaim.d.ts
type SystemClaim = {
  /**
   * Subject
   *
   * The subject of the token, typically the user's unique identifier (e.g., UUID)
   */
  sub: string;
  /**
   * Expiration Time
   *
   * Token expiration time (Unix timestamp, seconds)
   */
  exp: bigint;
  /**
   * Issued At
   *
   * Token issuance time (Unix timestamp, seconds)
   */
  iat: bigint;
  /**
   * Issuer
   *
   * Token issuer (optional), e.g., "oceaniam-auth"
   */
  iss: string | null;
  /**
   * Audience
   *
   * Token audience (optional), represents the intended recipient of the token
   */
  aud: Array<string> | null;
  /**
   * JWT ID
   *
   * Unique identifier for the token, used to prevent replay attacks
   */
  jti: string;
};
//#endregion
//#region src/types/SystemSigninRequest.d.ts
type SystemSigninRequest = {
  name: string;
  password: string;
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
export { AdministratorVO, ApplicationChallengeVO, ApplicationConfigurationVO, ApplicationDetailVO, ApplicationKeyVO, ApplicationUserVO, ApplicationUsersSortOrder, ApplicationVO, Argon2Configuration, AuthConfigurationVO, AuthVO, Claim, CreateAdministratorRequest, CreateAdministratorResponse, CreateApplicationRequest, CreateApplicationResponse, CreateApplicationUserRequest, CreateTenantRequest, ErrorResponse, GetApplicationConfigurationResponse, GetTenantsRequest, PasswordConfigurationVO, PatchAdministratorRequest, PatchApplicationConfigurationRequest, PatchApplicationRequest, PatchApplicationUserCredentialsRequest, PatchAuthConfigurationVO, PatchRegistrationConfigurationVO, PatchTenantRequest, PatchTokenConfigurationVO, PatchTotpConfigurationVO, RegistrationConfigurationVO, RotateKeyResponse, SecretVO, SigninChallenge, SigninRequest, SigninResponse, SigninResponseSchema, SignoutResponse, SignupResponse, Sqid, SystemClaim, SystemSigninRequest, SystemSigninResponse, TenantVO, TokenConfigurationVO, TokenDispatchMethod, TotpConfigurationVO };