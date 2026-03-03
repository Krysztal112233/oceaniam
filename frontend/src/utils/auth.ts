import { reactive } from "vue";

export type AuthUser = {
    username: string;
    displayName: string;
};

export const authState = reactive<{
    user: AuthUser | null;
    loading: boolean;
    error: string | null;
}>({
    user: null,
    loading: false,
    error: null,
});

const JWT_COOKIE_NAME = "jwt";

function cookieAttrs(): string[] {
    const attrs = ["Path=/", "SameSite=Lax"];
    if (typeof location !== "undefined" && location.protocol === "https:") {
        attrs.push("Secure");
    }
    return attrs;
}

function clearJwtCookie(): void {
    if (typeof document === "undefined") return;
    document.cookie = [
        `${JWT_COOKIE_NAME}=`,
        "Max-Age=0",
        ...cookieAttrs(),
    ].join("; ");
}

export function storage(jwt: string): void {
    if (typeof document === "undefined") return;
    document.cookie = [
        `${JWT_COOKIE_NAME}=${encodeURIComponent(jwt)}`,
        ...cookieAttrs(),
    ].join("; ");
}

// NOTE: THIS IS MOCKING AUTH.
export async function login(username: string, password: string): Promise<void> {
    authState.loading = true;
    authState.error = null;
    void username;
    void password;
}

export function logout(): void {
    clearJwtCookie();
    authState.user = null;
    authState.error = null;
}
