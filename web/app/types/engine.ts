export type EngineState = "idle" | "loading" | "ready" | "error"

export type RequestMethod = "GET"

export interface RequestHeader {
    name: string;
    value: string;
}

export interface PreparedRequest {
    key: string;
    url: string;
    method: RequestMethod;
    headers?: RequestHeader[];
}

export interface PreparedInput {
    key: string;
    normalizedInput: string;
    request: PreparedRequest;
}

export type ResourceKind = "image" | "video" | "animation" | "unknown"

export interface ResourceVariant {
    url: string;
    mimeType?: string;
    bitrate?: number;
    width?: number;
    height?: number;
}

export interface ResourceItem {
    id: string;
    kind: ResourceKind;
    preferredUrl: string;
    previewUrl?: string;
    width?: number;
    height?: number;
    durationMs?: number;
    variants: ResourceVariant[];
}

export interface ResourceBundle {
    schemaVersion: number;
    sourceKey: string;
    resources: ResourceItem[];
}

export interface EngineErrorShape {
    code?: string;
    message?: string;
}

export interface EngineRuntime {
    prepare(input: string): PreparedInput;

    complete(input: string, status: number, body: Uint8Array): ResourceBundle;
}
