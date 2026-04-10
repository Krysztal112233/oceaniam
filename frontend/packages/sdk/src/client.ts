import { type ApplicationVO } from "./types/ApplicationVO";
import { type ApplicationDetailVO } from "./types/ApplicationDetailVO";
import { type ApplicationUserVO } from "./types/ApplicationUserVO";
import { type CreateApplicationRequest } from "./types/CreateApplicationRequest";
import { type CreateApplicationResponse } from "./types/CreateApplicationResponse";
import { type CreateApplicationUserRequest } from "./types/CreateApplicationUserRequest";
import { type GetApplicationConfigurationResponse } from "./types/GetApplicationConfigurationResponse";
import { type PatchApplicationRequest } from "./types/PatchApplicationRequest";
import { type PatchApplicationConfigurationRequest } from "./types/PatchApplicationConfigurationRequest";
import { type PatchApplicationUserCredentialsRequest } from "./types/PatchApplicationUserCredentialsRequest";
import { type CreateTenantRequest } from "./types/CreateTenantRequest";
import { type PagedResponse } from "./pagination";
import { type Sqid } from "./types/Sqid";
import { type SignoutResponse } from "./types/SignoutResponse";
import { type SystemSigninRequest } from "./types/SystemSigninRequest";
import { type SystemSigninResponse } from "./types/SystemSigninResponse";
import { type TenantVO } from "./types/TenantVO";
import { type TokenDispatchMethod } from "./types/TokenDispatchMethod";
import { type SecretVO } from "./types/SecretVO";

export type TokenGetter = () =>
    | string
    | null
    | undefined
    | Promise<string | null | undefined>;

export type GetApplicationsQuery = {
    tenant_id: Sqid;
} & PaginationQuery;

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
export type GetApplicationUsersQuery = PaginationQuery;
export type SearchApplicationUsersQuery = {
    by_nickname?: string;
    by_email?: string;
    by_id?: string;
    by_phone?: string;
};
export type GetSecretsQuery = PaginationQuery;

type TenantScopedCreateApplicationRequest = Omit<
    CreateApplicationRequest,
    "tenant_id"
>;

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
            throw new Error(
                "System signin succeeded but response JSON is invalid.",
            );
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

export async function doSystemSignout(
    jwt: string,
    baseUrl?: string,
): Promise<SignoutResponse> {
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
            throw new Error(
                "System signout succeeded but response JSON is invalid.",
            );
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
            throw new Error(
                "System token refresh succeeded but response JSON is invalid.",
            );
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
        tenant: (tenantId: string): string =>
            `/tenants/${encodeURIComponent(tenantId)}`,
        tenantUsers: (tenantId: string): string =>
            `/tenants/${encodeURIComponent(tenantId)}/users`,

        // Applications (backend: endpoints/applications.rs)
        applications: (tenantId: string): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications`,
        application: (tenantId: string, applicationId: string): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}`,
        applicationConfiguration: (
            tenantId: string,
            applicationId: string,
        ): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/configuration`,
        applicationJwks: (applicationId: string): string =>
            `/applications/${encodeURIComponent(applicationId)}/.well-known/jwks.json`,

        // ApplicationUsers
        applicationUsers: (tenantId: string, applicationId: string): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users`,
        applicationUsersCreate: (
            tenantId: string,
            applicationId: string,
        ): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users`,
        applicationUsersSearch: (
            tenantId: string,
            applicationId: string,
        ): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/search`,
        applicationUserCredentials: (
            tenantId: string,
            applicationId: string,
            userId: string,
        ): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}/credentials`,

        // ApplicationUserAuthentication
        applicationAuthTokens: (
            tenantId: string,
            applicationId: string,
        ): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/tokens`,
        applicationAuthTokensRefresh: (
            tenantId: string,
            applicationId: string,
        ): string =>
            `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/tokens/refresh`,

        // Secrets (backend: endpoints/secrets.rs)
        secrets: "/secrets",
        secret: (secretId: string): string =>
            `/secrets/${encodeURIComponent(secretId)}`,
    } as const;

    private baseUrl: string;
    private tokenGetter?: TokenGetter;
    private onUnauthorized?: () => Promise<string | null | undefined>;

    constructor({
        baseUrl,
        tokenGetter,
        onUnauthorized,
    }: OceanIamClientConfig) {
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
        systemSignin: (): string =>
            this.buildUrl(OceanIamClient.PATHS.systemAuthTokens),
        systemSignout: (): string =>
            this.buildUrl(OceanIamClient.PATHS.systemAuthTokens),
        systemSignup: (): string =>
            this.buildUrl(OceanIamClient.PATHS.systemAuthUsers),
        systemRefreshToken: (): string =>
            this.buildUrl(OceanIamClient.PATHS.systemAuthTokensRefresh),

        // Tenants
        tenants: (): string => this.buildUrl(OceanIamClient.PATHS.tenants),
        tenant: (tenantId: string): string =>
            this.buildUrl(OceanIamClient.PATHS.tenant(tenantId)),
        tenantUsers: (tenantId: string): string =>
            this.buildUrl(OceanIamClient.PATHS.tenantUsers(tenantId)),

        // Applications
        applications: (tenantId: string): string =>
            this.buildUrl(OceanIamClient.PATHS.applications(tenantId)),
        application: (tenantId: string, applicationId: string): string =>
            this.buildUrl(
                OceanIamClient.PATHS.application(tenantId, applicationId),
            ),
        applicationConfiguration: (
            tenantId: string,
            applicationId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationConfiguration(
                    tenantId,
                    applicationId,
                ),
            ),
        applicationJwks: (applicationId: string): string =>
            this.buildUrl(OceanIamClient.PATHS.applicationJwks(applicationId)),

        // ApplicationUsers
        applicationUsers: (tenantId: string, applicationId: string): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationUsers(tenantId, applicationId),
            ),
        applicationUsersCreate: (
            tenantId: string,
            applicationId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationUsersCreate(
                    tenantId,
                    applicationId,
                ),
            ),
        applicationUsersSearch: (
            tenantId: string,
            applicationId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationUsersSearch(
                    tenantId,
                    applicationId,
                ),
            ),
        applicationUserCredentials: (
            tenantId: string,
            applicationId: string,
            userId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationUserCredentials(
                    tenantId,
                    applicationId,
                    userId,
                ),
            ),

        // ApplicationUserAuthentication
        applicationUserSignin: (
            tenantId: string,
            applicationId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationAuthTokens(
                    tenantId,
                    applicationId,
                ),
            ),
        applicationUserSignout: (
            tenantId: string,
            applicationId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationAuthTokens(
                    tenantId,
                    applicationId,
                ),
            ),
        applicationUserRefreshToken: (
            tenantId: string,
            applicationId: string,
        ): string =>
            this.buildUrl(
                OceanIamClient.PATHS.applicationAuthTokensRefresh(
                    tenantId,
                    applicationId,
                ),
            ),

        // Secrets
        secrets: (): string => this.buildUrl(OceanIamClient.PATHS.secrets),
        secret: (secretId: string): string =>
            this.buildUrl(OceanIamClient.PATHS.secret(secretId)),
    } as const;

    public async getTenants(
        query?: GetTenantsQuery,
    ): Promise<PagedResponse<TenantVO>> {
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

    public async getApplications(
        query: GetApplicationsQuery,
    ): Promise<PagedResponse<ApplicationVO>> {
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

    public async deleteApplication(
        tenantId: string,
        applicationId: string,
    ): Promise<void> {
        await this.request<unknown>({
            method: "DELETE",
            url: this.endpoints.application(tenantId, applicationId),
        });
    }

    public async getApplicationUsers(
        tenantId: string,
        applicationId: string,
        query?: GetApplicationUsersQuery,
    ): Promise<PagedResponse<ApplicationUserVO>> {
        const { page, per_page } = query ?? {};
        return this.request<PagedResponse<ApplicationUserVO>>({
            method: "GET",
            url: this.endpoints.applicationUsers(tenantId, applicationId),
            query: { page, per_page },
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
        const { by_nickname, by_email, by_id, by_phone } = query;
        return this.request<PagedResponse<ApplicationUserVO>>({
            method: "GET",
            url: this.endpoints.applicationUsersSearch(tenantId, applicationId),
            query: { by_nickname, by_email, by_id, by_phone },
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
            url: this.endpoints.applicationUserCredentials(
                tenantId,
                applicationId,
                userId,
            ),
            body: req,
        });
    }

    public async getApplicationConfiguration(
        tenantId: string,
        applicationId: string,
    ): Promise<GetApplicationConfigurationResponse> {
        return this.request<GetApplicationConfigurationResponse>({
            method: "GET",
            url: this.endpoints.applicationConfiguration(
                tenantId,
                applicationId,
            ),
        });
    }

    public async patchApplicationConfiguration(
        tenantId: string,
        applicationId: string,
        req: PatchApplicationConfigurationRequest,
    ): Promise<void> {
        await this.request<unknown>({
            method: "PATCH",
            url: this.endpoints.applicationConfiguration(
                tenantId,
                applicationId,
            ),
            body: req,
        });
    }

    public async createSecret(): Promise<SecretVO> {
        return this.request<SecretVO>({
            method: "POST",
            url: this.endpoints.secrets(),
        });
    }

    public async getSecrets(
        query?: GetSecretsQuery,
    ): Promise<PagedResponse<SecretVO>> {
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

    public async getApplicationJwks(applicationId: string): Promise<unknown> {
        return this.request<unknown>({
            method: "GET",
            url: this.endpoints.applicationJwks(applicationId),
            auth: "none",
        });
    }

    private buildUrl(path: string): string {
        if (!this.baseUrl) return path;

        const base = this.baseUrl.replace(/\/+$/, "");
        const normalizedPath = path.startsWith("/") ? path : `/${path}`;
        return `${base}${normalizedPath}`;
    }

    private buildUrlWithQuery(
        url: string,
        query?: Record<string, QueryValue>,
    ): string {
        if (!query) return url;

        const params = new URLSearchParams();
        for (const [key, value] of Object.entries(query)) {
            if (value === undefined || value === null) continue;
            params.set(
                key,
                typeof value === "bigint" ? value.toString() : String(value),
            );
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

        if (
            result.status === "unauthorized" &&
            this.onUnauthorized &&
            opts.auth !== "none"
        ) {
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
        const {
            method,
            url,
            query,
            body,
            auth = "required",
            headers: extraHeaders,
        } = opts;

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
                throw new Error(
                    `Failed to parse JSON response from ${method} ${finalUrl}: ${String(e)}`,
                );
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
