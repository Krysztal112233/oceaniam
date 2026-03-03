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

export async function login(
    _username: string,
    _password: string,
): Promise<void> {
    return;
}

export async function logout(): Promise<void> {
    return;
}
