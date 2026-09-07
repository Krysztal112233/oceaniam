import createClient, { type Middleware } from "openapi-fetch";

import type { paths } from "./schema.js";

export type TokenGetter = () =>
  string | null | undefined | Promise<string | null | undefined>;

export interface OceanIamClientOptions {
  baseUrl?: string;
  getToken?: TokenGetter;
  beforeRequest?: () => unknown | Promise<unknown>;
  onUnauthorized?: () =>
    string | null | undefined | Promise<string | null | undefined>;
  fetch?: typeof globalThis.fetch;
}

export interface ApiResult<T> {
  data?: T;
  error?: unknown;
  response: Response;
}

export class OceanIamApiError extends Error {
  readonly status: number;
  readonly payload: unknown;

  constructor(response: Response, payload: unknown) {
    super(errorMessage(response, payload));
    this.name = "OceanIamApiError";
    this.status = response.status;
    this.payload = payload;
  }
}

function errorMessage(response: Response, payload: unknown): string {
  if (payload && typeof payload === "object" && "msg" in payload) {
    const msg = (payload as { msg?: unknown }).msg;
    if (typeof msg === "string" && msg.trim()) return msg;
  }
  return `${response.status} ${response.statusText}`.trim();
}

export function unwrap<T>(result: ApiResult<T>): NonNullable<T> {
  if (result.data !== undefined) return result.data as NonNullable<T>;
  throw new OceanIamApiError(result.response, result.error);
}

function withBearer(
  request: Request,
  token: string | null | undefined,
): Request {
  const headers = new Headers(request.headers);
  if (token?.trim()) headers.set("Authorization", `Bearer ${token.trim()}`);
  else headers.delete("Authorization");
  return new Request(request, { headers });
}

async function isAuthenticationFailure(response: Response): Promise<boolean> {
  // OceanIAM reports a missing credential as 203 and an invalid credential as
  // a 400 error payload; normalize both alongside conventional 401 responses.
  if (response.status === 401 || response.status === 203) return true;
  if (response.status !== 400) return false;

  try {
    const payload: unknown = await response.clone().json();
    return (
      payload !== null &&
      typeof payload === "object" &&
      "msg" in payload &&
      (payload as { msg?: unknown }).msg === "authentication failed"
    );
  } catch {
    return false;
  }
}

function normalizedAuthenticationFailure(response: Response): Response {
  if (response.status === 401) return response;
  return new Response(response.body, {
    headers: response.headers,
    status: 401,
    statusText: "Unauthorized",
  });
}

export function createOceanIamClient(options: OceanIamClientOptions = {}) {
  const nativeFetch = options.fetch ?? globalThis.fetch.bind(globalThis);

  const retryingFetch: typeof globalThis.fetch = async (input, init) => {
    await options.beforeRequest?.();
    let request = new Request(input, init);
    request = withBearer(request, await options.getToken?.());

    let response = await nativeFetch(request);
    if (!(await isAuthenticationFailure(response))) return response;
    if (!options.onUnauthorized)
      return normalizedAuthenticationFailure(response);

    const refreshedToken = await options.onUnauthorized();
    if (!refreshedToken) return normalizedAuthenticationFailure(response);

    request = withBearer(request, refreshedToken);
    response = await nativeFetch(request);
    if (await isAuthenticationFailure(response))
      return normalizedAuthenticationFailure(response);
    return response;
  };

  const client = createClient<paths>({
    baseUrl: options.baseUrl ?? "/api",
    fetch: retryingFetch,
  });

  const authMiddleware: Middleware = {
    async onRequest({ request }) {
      return withBearer(request, await options.getToken?.());
    },
  };
  client.use(authMiddleware);
  return client;
}

export async function systemSignin(
  baseUrl: string,
  name: string,
  password: string,
): Promise<string> {
  const client = createClient<paths>({ baseUrl });
  const result = await client.POST("/auth/tokens", {
    headers: { "X-OceanIAM-Token-Dispatch": "json" },
    body: { name, password },
  });
  const payload = unwrap(result);
  if (!("jwt" in payload) || typeof payload.jwt !== "string") {
    throw new Error("Authentication response did not include a JWT.");
  }
  return payload.jwt;
}

export async function systemRefresh(
  baseUrl: string,
  token: string,
): Promise<string> {
  const client = createClient<paths>({ baseUrl });
  const result = await client.POST("/auth/tokens/refresh", {
    params: {
      header: {
        Authorization: `Bearer ${token}`,
        "X-OceanIAM-Token-Dispatch": "json",
      },
    },
  });
  const payload = unwrap(result);
  if (!payload || !("jwt" in payload) || typeof payload.jwt !== "string") {
    throw new Error("Token refresh response did not include a JWT.");
  }
  return payload.jwt;
}

export async function systemSignout(
  baseUrl: string,
  token: string,
): Promise<void> {
  const client = createClient<paths>({ baseUrl });
  unwrap(
    await client.DELETE("/auth/tokens", {
      params: { header: { Authorization: `Bearer ${token}` } },
    }),
  );
}
