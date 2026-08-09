import type { ResourceBundle } from "./engine";

export type RequestMethod = "GET";

export interface RequestHeader {
    name: string;
    value: string;
}

export interface RetryPolicy {
    maxRetries: number;
    delaysMs: number[];
    retryStatuses: number[];
}

export interface PreparedRequest {
    routeKey: string;
    url: string;
    method: RequestMethod;
    headers?: RequestHeader[];
    retryPolicy: RetryPolicy;
}

export type RuntimeProfile = "browser" | "native";

export interface ResolutionOptions {
    profile: RuntimeProfile;
    gatewayEndpoint?: string;
}

// The browser deliberately treats the serialized Rust session as opaque control data.
export type ResolutionSession = unknown;

export interface ResolveFailure {
    code: string;
    message?: string;
}

export type ResolutionStep =
        | {
    kind: "request";
    session: ResolutionSession;
    request: PreparedRequest;
    sourceKey: string;
    normalizedInput: string;
}
        | {
    kind: "resolved";
    result: ResourceBundle;
}
        | {
    kind: "failed";
    error: ResolveFailure;
};

export type TransportFailureKind = "network" | "access_blocked" | "timeout";
