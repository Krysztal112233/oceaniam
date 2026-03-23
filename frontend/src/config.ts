export interface AppConfig {
    systemBaseUrl: string;
}

const env = import.meta.env;

export const appConfig: AppConfig = {
    systemBaseUrl: env.VITE_BASE_URL?.trim() || "/api",
};
