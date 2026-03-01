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

function storage(jwt: string) {}

// NOTE: THIS IS MOCKING AUTH.
export async function login(username: string, password: string): Promise<void> {
    authState.loading = true;
    authState.error = null;
}

export function logout(): void {
    authState.user = null;
    authState.error = null;
}
