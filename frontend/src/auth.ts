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

function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

// NOTE: THIS IS MOCKING AUTH.
export async function login(username: string, password: string): Promise<void> {
    authState.loading = true;
    authState.error = null;

    try {
        await sleep(350);

        if (!username.trim() || !password) {
            throw new Error("请输入账号和密码");
        }

        authState.user = {
            username: username.trim(),
            displayName: username.trim(),
        };
    } catch (err) {
        authState.user = null;
        authState.error = err instanceof Error ? err.message : "登录失败";
        throw err;
    } finally {
        authState.loading = false;
    }
}

export function logout(): void {
    authState.user = null;
    authState.error = null;
}

