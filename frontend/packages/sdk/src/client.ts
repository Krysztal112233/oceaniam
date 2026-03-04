import { type SystemSigninResponse } from "./types/SystemSigninResponse";
import { type SystemSigninRequest } from "./types/SystemSigninRequest";
import { type TokenDispatchMethod } from "./types/TokenDispatchMethod";

export async function doSystemAuth(
    name: string,
    password: string,
    baseUrl?: string,
    dispatch?: TokenDispatchMethod,
) {}

export interface OceanIamClientConfig {
    baseUrl?: string;
    jwt: string;
}

export class OceanIamClient {
    private baseUrl: string;
    private jwt: string;

    constructor({ baseUrl, jwt }: OceanIamClientConfig) {
        this.jwt = jwt;
        this.baseUrl = baseUrl ?? "";
    }
}
