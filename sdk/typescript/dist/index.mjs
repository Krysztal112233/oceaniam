function toTokenDispatchHeader(method) {
	if (!method) return void 0;
	if (method === "Cookie") return "cookie";
	if (method === "Json") return "json";
	if (method === "Both") return "both";
}
function resolveSystemUrl(path, baseUrl) {
	const normalizedBase = baseUrl?.trim() ?? "";
	if (!normalizedBase) {
		if (typeof window === "undefined") throw new Error(`Relative URL is not supported in Node.js fetch: ${path}. Provide baseUrl (e.g. http://127.0.0.1:3000).`);
		return path;
	}
	return `${normalizedBase.replace(/\/+$/, "")}${path.startsWith("/") ? path : `/${path}`}`;
}
async function doSystemAuth(name, password, baseUrl, dispatch) {
	if (typeof fetch !== "function") throw new Error("Global fetch is not available in this runtime. Use Node.js 18+ or provide a fetch polyfill.");
	const headers = {
		Accept: "application/json",
		"Content-Type": "application/json"
	};
	const tokenDispatch = toTokenDispatchHeader(dispatch);
	if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;
	const res = await fetch(resolveSystemUrl("/auth/tokens", baseUrl), {
		method: "POST",
		headers,
		body: JSON.stringify({
			name,
			password
		})
	});
	const text = await res.text();
	if (res.ok) try {
		return JSON.parse(text);
	} catch {
		throw new Error("System signin succeeded but response JSON is invalid.");
	}
	let msg = text || `${res.status} ${res.statusText}`;
	try {
		const json = JSON.parse(text);
		if (json && typeof json === "object" && "msg" in json && typeof json.msg === "string") msg = json.msg;
	} catch {}
	throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
}
async function doSystemSignout(jwt, baseUrl) {
	if (!jwt.trim()) throw new Error("JWT is required for system signout.");
	if (typeof fetch !== "function") throw new Error("Global fetch is not available in this runtime. Use Node.js 18+ or provide a fetch polyfill.");
	const res = await fetch(resolveSystemUrl("/auth/tokens", baseUrl), {
		method: "DELETE",
		headers: {
			Accept: "application/json",
			Authorization: `Bearer ${jwt.trim()}`
		}
	});
	const text = await res.text();
	if (res.ok && res.status !== 203) {
		if (!text) return { msg: "" };
		try {
			return JSON.parse(text);
		} catch {
			throw new Error("System signout succeeded but response JSON is invalid.");
		}
	}
	let msg = text || `${res.status} ${res.statusText}`;
	try {
		const json = JSON.parse(text);
		if (json && typeof json === "object" && "msg" in json && typeof json.msg === "string") msg = json.msg;
	} catch {}
	throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
}
async function doSystemRefreshToken(jwt, baseUrl, dispatch) {
	if (!jwt.trim()) throw new Error("JWT is required for system token refresh.");
	if (typeof fetch !== "function") throw new Error("Global fetch is not available in this runtime. Use Node.js 18+ or provide a fetch polyfill.");
	const headers = {
		Accept: "application/json",
		Authorization: `Bearer ${jwt.trim()}`
	};
	const tokenDispatch = toTokenDispatchHeader(dispatch);
	if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;
	const res = await fetch(resolveSystemUrl("/auth/tokens/refresh", baseUrl), {
		method: "POST",
		headers
	});
	const text = await res.text();
	if (res.ok) try {
		return JSON.parse(text);
	} catch {
		throw new Error("System token refresh succeeded but response JSON is invalid.");
	}
	let msg = text || `${res.status} ${res.statusText}`;
	try {
		const json = JSON.parse(text);
		if (json && typeof json === "object" && "msg" in json && typeof json.msg === "string") msg = json.msg;
	} catch {}
	throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
}
var OceanIamClient = class OceanIamClient {
	static PATHS = {
		root: "/",
		systemAuthTokens: "/auth/tokens",
		systemAuthUsers: "/auth/users",
		systemAuthTokensRefresh: "/auth/tokens/refresh",
		tenants: "/tenants",
		tenant: (tenantId) => `/tenants/${encodeURIComponent(tenantId)}`,
		tenantUsers: (tenantId) => `/tenants/${encodeURIComponent(tenantId)}/users`,
		administrators: "/administrators",
		administrator: (targetId) => `/administrators/${encodeURIComponent(targetId)}`,
		applications: (tenantId) => `/tenants/${encodeURIComponent(tenantId)}/applications`,
		application: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}`,
		applicationConfiguration: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/configuration`,
		applicationJwks: (applicationId) => `/applications/${encodeURIComponent(applicationId)}/.well-known/jwks.json`,
		applicationUsers: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users`,
		applicationUsersCreate: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users`,
		applicationUsersSearch: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/search`,
		applicationUserCredentials: (tenantId, applicationId, userId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}/credentials`,
		applicationUser: (tenantId, applicationId, userId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/users/${encodeURIComponent(userId)}`,
		applicationChallenge: (tenantId, applicationId, challengeId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/challenges/${encodeURIComponent(challengeId)}`,
		applicationKeys: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/keys`,
		applicationKey: (tenantId, applicationId, keyId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/keys/${encodeURIComponent(keyId)}`,
		applicationTokens: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/tokens`,
		applicationTokensRefresh: (tenantId, applicationId) => `/tenants/${encodeURIComponent(tenantId)}/applications/${encodeURIComponent(applicationId)}/tokens/refresh`
	};
	baseUrl;
	tokenGetter;
	onUnauthorized;
	constructor({ baseUrl, tokenGetter, onUnauthorized }) {
		this.baseUrl = baseUrl?.trim() ?? "";
		this.tokenGetter = tokenGetter;
		this.onUnauthorized = onUnauthorized;
	}
	endpoints = {
		root: () => this.buildUrl(OceanIamClient.PATHS.root),
		systemSignin: () => this.buildUrl(OceanIamClient.PATHS.systemAuthTokens),
		systemSignout: () => this.buildUrl(OceanIamClient.PATHS.systemAuthTokens),
		systemSignup: () => this.buildUrl(OceanIamClient.PATHS.systemAuthUsers),
		systemRefreshToken: () => this.buildUrl(OceanIamClient.PATHS.systemAuthTokensRefresh),
		tenants: () => this.buildUrl(OceanIamClient.PATHS.tenants),
		tenant: (tenantId) => this.buildUrl(OceanIamClient.PATHS.tenant(tenantId)),
		tenantUsers: (tenantId) => this.buildUrl(OceanIamClient.PATHS.tenantUsers(tenantId)),
		administrators: () => this.buildUrl(OceanIamClient.PATHS.administrators),
		administrator: (targetId) => this.buildUrl(OceanIamClient.PATHS.administrator(targetId)),
		applications: (tenantId) => this.buildUrl(OceanIamClient.PATHS.applications(tenantId)),
		application: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.application(tenantId, applicationId)),
		applicationConfiguration: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationConfiguration(tenantId, applicationId)),
		applicationJwks: (applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationJwks(applicationId)),
		applicationUsers: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationUsers(tenantId, applicationId)),
		applicationUsersCreate: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationUsersCreate(tenantId, applicationId)),
		applicationUsersSearch: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationUsersSearch(tenantId, applicationId)),
		applicationUserCredentials: (tenantId, applicationId, userId) => this.buildUrl(OceanIamClient.PATHS.applicationUserCredentials(tenantId, applicationId, userId)),
		applicationUser: (tenantId, applicationId, userId) => this.buildUrl(OceanIamClient.PATHS.applicationUser(tenantId, applicationId, userId)),
		applicationChallenge: (tenantId, applicationId, challengeId) => this.buildUrl(OceanIamClient.PATHS.applicationChallenge(tenantId, applicationId, challengeId)),
		applicationKeys: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationKeys(tenantId, applicationId)),
		applicationKey: (tenantId, applicationId, keyId) => this.buildUrl(OceanIamClient.PATHS.applicationKey(tenantId, applicationId, keyId)),
		applicationUserSignin: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationTokens(tenantId, applicationId)),
		applicationUserSignout: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationTokens(tenantId, applicationId)),
		applicationUserRefreshToken: (tenantId, applicationId) => this.buildUrl(OceanIamClient.PATHS.applicationTokensRefresh(tenantId, applicationId)),
		secrets: () => this.buildUrl(OceanIamClient.PATHS.secrets),
		secret: (secretId) => this.buildUrl(OceanIamClient.PATHS.secret(secretId))
	};
	async getTenants(query) {
		const { page, per_page } = query ?? {};
		return this.request({
			method: "GET",
			url: this.endpoints.tenants(),
			query: {
				page,
				per_page
			}
		});
	}
	async getTenant(tenantId) {
		return this.request({
			method: "GET",
			url: this.endpoints.tenant(tenantId)
		});
	}
	async createTenant(req) {
		return this.request({
			method: "POST",
			url: this.endpoints.tenants(),
			body: req
		});
	}
	async deleteTenant(tenantId) {
		await this.request({
			method: "DELETE",
			url: this.endpoints.tenant(tenantId)
		});
	}
	async patchTenant(tenantId, req) {
		return this.request({
			method: "PATCH",
			url: this.endpoints.tenant(tenantId),
			body: req
		});
	}
	async getTenantUsers(tenantId, query) {
		const { page, per_page } = query ?? {};
		return this.request({
			method: "GET",
			url: this.endpoints.tenantUsers(tenantId),
			query: {
				page,
				per_page
			}
		});
	}
	async getApplications(query) {
		const { tenant_id, page, per_page } = query;
		return this.request({
			method: "GET",
			url: this.endpoints.applications(tenant_id),
			query: {
				page,
				per_page
			}
		});
	}
	async getApplication(tenantId, applicationId) {
		return this.request({
			method: "GET",
			url: this.endpoints.application(tenantId, applicationId)
		});
	}
	async createApplication(tenantId, req) {
		return this.request({
			method: "POST",
			url: this.endpoints.applications(tenantId),
			body: req
		});
	}
	async patchApplication(tenantId, applicationId, req) {
		return this.request({
			method: "PATCH",
			url: this.endpoints.application(tenantId, applicationId),
			body: req
		});
	}
	async deleteApplication(tenantId, applicationId) {
		await this.request({
			method: "DELETE",
			url: this.endpoints.application(tenantId, applicationId)
		});
	}
	async getApplicationUser(tenantId, applicationId, userId) {
		return this.request({
			method: "GET",
			url: this.endpoints.applicationUser(tenantId, applicationId, userId)
		});
	}
	async getApplicationUsers(tenantId, applicationId, query) {
		const { page, per_page, sort_order } = query ?? {};
		return this.request({
			method: "GET",
			url: this.endpoints.applicationUsers(tenantId, applicationId),
			query: {
				page,
				per_page,
				sort_order
			}
		});
	}
	async createApplicationUser(tenantId, applicationId, req) {
		return this.request({
			method: "POST",
			url: this.endpoints.applicationUsersCreate(tenantId, applicationId),
			body: req
		});
	}
	async searchApplicationUsers(tenantId, applicationId, query) {
		const { by_nickname, by_email, by_id, by_phone, page, per_page, sort_order } = query;
		return this.request({
			method: "GET",
			url: this.endpoints.applicationUsersSearch(tenantId, applicationId),
			query: {
				by_nickname,
				by_email,
				by_id,
				by_phone,
				page,
				per_page,
				sort_order
			}
		});
	}
	async patchApplicationUserCredentials(tenantId, applicationId, userId, req) {
		return this.request({
			method: "PATCH",
			url: this.endpoints.applicationUserCredentials(tenantId, applicationId, userId),
			body: req
		});
	}
	async applicationUserSignin(tenantId, applicationId, auth, dispatch) {
		const headers = {};
		const tokenDispatch = toTokenDispatchHeader(dispatch);
		if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;
		return this.request({
			method: "POST",
			url: this.endpoints.applicationUserSignin(tenantId, applicationId),
			body: auth,
			headers
		});
	}
	async applicationUserSignout(tenantId, applicationId) {
		return this.request({
			method: "DELETE",
			url: this.endpoints.applicationUserSignout(tenantId, applicationId)
		});
	}
	async applicationUserRefreshToken(tenantId, applicationId, dispatch) {
		const headers = {};
		const tokenDispatch = toTokenDispatchHeader(dispatch);
		if (tokenDispatch) headers["X-OceanIAM-Token-Dispatch"] = tokenDispatch;
		return this.request({
			method: "POST",
			url: this.endpoints.applicationUserRefreshToken(tenantId, applicationId),
			headers
		});
	}
	async getApplicationConfiguration(tenantId, applicationId) {
		return this.request({
			method: "GET",
			url: this.endpoints.applicationConfiguration(tenantId, applicationId)
		});
	}
	async patchApplicationConfiguration(tenantId, applicationId, req) {
		await this.request({
			method: "PATCH",
			url: this.endpoints.applicationConfiguration(tenantId, applicationId),
			body: req
		});
	}
	async createSecret() {
		return this.request({
			method: "POST",
			url: this.endpoints.secrets()
		});
	}
	async getSecrets(query) {
		const { page, per_page } = query ?? {};
		return this.request({
			method: "GET",
			url: this.endpoints.secrets(),
			query: {
				page,
				per_page
			}
		});
	}
	async getSecret(secretId) {
		return this.request({
			method: "GET",
			url: this.endpoints.secret(secretId)
		});
	}
	async deleteSecret(secretId) {
		await this.request({
			method: "DELETE",
			url: this.endpoints.secret(secretId)
		});
	}
	async getApplicationJwks(applicationId) {
		return this.request({
			method: "GET",
			url: this.endpoints.applicationJwks(applicationId),
			auth: "none"
		});
	}
	async systemSignup(name, password) {
		await this.request({
			method: "POST",
			url: this.endpoints.systemSignup(),
			body: {
				name,
				password
			}
		});
	}
	async getAdministrators(query) {
		const { page, per_page } = query ?? {};
		return this.request({
			method: "GET",
			url: this.endpoints.administrators(),
			query: {
				page,
				per_page
			}
		});
	}
	async createAdministrator(req) {
		return this.request({
			method: "POST",
			url: this.endpoints.administrators(),
			body: req
		});
	}
	async patchAdministrator(targetId, req) {
		return this.request({
			method: "PATCH",
			url: this.endpoints.administrator(targetId),
			body: req
		});
	}
	async getApplicationChallenge(tenantId, applicationId, challengeId) {
		return this.request({
			method: "GET",
			url: this.endpoints.applicationChallenge(tenantId, applicationId, challengeId)
		});
	}
	async getApplicationKeys(tenantId, applicationId) {
		return this.request({
			method: "GET",
			url: this.endpoints.applicationKeys(tenantId, applicationId)
		});
	}
	async rotateApplicationKey(tenantId, applicationId) {
		return this.request({
			method: "POST",
			url: this.endpoints.applicationKeys(tenantId, applicationId)
		});
	}
	async revokeApplicationKey(tenantId, applicationId, keyId) {
		await this.request({
			method: "DELETE",
			url: this.endpoints.applicationKey(tenantId, applicationId, keyId)
		});
	}
	async submitApplicationChallenge(tenantId, applicationId, challengeId, payload) {
		return this.request({
			method: "POST",
			url: this.endpoints.applicationChallenge(tenantId, applicationId, challengeId),
			body: payload
		});
	}
	buildUrl(path) {
		if (!this.baseUrl) return path;
		return `${this.baseUrl.replace(/\/+$/, "")}${path.startsWith("/") ? path : `/${path}`}`;
	}
	buildUrlWithQuery(url, query) {
		if (!query) return url;
		const params = new URLSearchParams();
		for (const [key, value] of Object.entries(query)) {
			if (value === void 0 || value === null) continue;
			params.set(key, typeof value === "bigint" ? value.toString() : String(value));
		}
		const qs = params.toString();
		if (!qs) return url;
		return url.includes("?") ? `${url}&${qs}` : `${url}?${qs}`;
	}
	async request(opts) {
		const result = await this._doRequest(opts);
		if (result.status === "unauthorized" && this.onUnauthorized && opts.auth !== "none") {
			const newToken = await this.onUnauthorized();
			if (newToken?.trim()) return this._doRequestWithToken(opts, newToken.trim());
		}
		if (result.status === "unauthorized" || result.status === "error") throw result.error;
		return result.value;
	}
	async _doRequest(opts) {
		const { method, url, query, body, auth = "required", headers: extraHeaders } = opts;
		const finalUrl = this.buildUrlWithQuery(url, query);
		const headers = {
			Accept: "application/json",
			...extraHeaders
		};
		if (auth !== "none") {
			const token = (await this.tokenGetter?.())?.trim();
			if (!token) return {
				status: "error",
				error: /* @__PURE__ */ new Error("Missing auth token. Provide OceanIamClientConfig.tokenGetter that returns a non-empty token string.")
			};
			headers.Authorization = `Bearer ${token}`;
		}
		const hasBody = body !== void 0;
		if (hasBody) headers["Content-Type"] = "application/json";
		const res = await fetch(finalUrl, {
			method,
			headers,
			body: hasBody ? JSON.stringify(body) : void 0
		});
		if (res.status === 401 || res.status === 203) return {
			status: "unauthorized",
			error: /* @__PURE__ */ new Error(`HTTP ${res.status}`)
		};
		if (res.ok) {
			const text = await res.text();
			if (!text) return {
				status: "ok",
				value: void 0
			};
			try {
				return {
					status: "ok",
					value: JSON.parse(text)
				};
			} catch (e) {
				return {
					status: "error",
					error: /* @__PURE__ */ new Error(`Failed to parse JSON response from ${method} ${finalUrl}: ${String(e)}`)
				};
			}
		}
		const errorText = await res.text();
		const msg = (() => {
			try {
				const json = JSON.parse(errorText);
				if (json && typeof json === "object" && "msg" in json && typeof json.msg === "string") return json.msg;
			} catch {}
			return errorText || `${res.status} ${res.statusText}`;
		})();
		return {
			status: "error",
			error: /* @__PURE__ */ new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`)
		};
	}
	async _doRequestWithToken(opts, token) {
		const { method, url, query, body, headers: extraHeaders } = opts;
		const finalUrl = this.buildUrlWithQuery(url, query);
		const headers = {
			Accept: "application/json",
			Authorization: `Bearer ${token}`,
			...extraHeaders
		};
		const hasBody = body !== void 0;
		if (hasBody) headers["Content-Type"] = "application/json";
		const res = await fetch(finalUrl, {
			method,
			headers,
			body: hasBody ? JSON.stringify(body) : void 0
		});
		if (res.ok && res.status !== 203) {
			const text = await res.text();
			if (!text) return void 0;
			try {
				return JSON.parse(text);
			} catch (e) {
				throw new Error(`Failed to parse JSON response from ${method} ${finalUrl}: ${String(e)}`);
			}
		}
		const errorText = await res.text();
		const msg = (() => {
			try {
				const json = JSON.parse(errorText);
				if (json && typeof json === "object" && "msg" in json && typeof json.msg === "string") return json.msg;
			} catch {}
			return errorText || `${res.status} ${res.statusText}`;
		})();
		throw new Error(`HTTP ${res.status} ${res.statusText}: ${msg}`);
	}
};
export { OceanIamClient, doSystemAuth, doSystemRefreshToken, doSystemSignout };
