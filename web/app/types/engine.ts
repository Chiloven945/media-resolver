import type {
    ResolutionOptions,
    ResolutionSession,
    ResolutionStep,
    TransportFailureKind
} from "./resolution";

export type EngineState = "idle" | "loading" | "ready" | "error";

export type ResourceKind = "image" | "video" | "animation" | "unknown";

export interface ResourceVariant {
    url: string;
    mimeType?: string;
    container?: string;
    codec?: string;
    bitrate?: number;
    sizeBytes?: number;
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
    start(input: string, options: ResolutionOptions): ResolutionStep;

    respond(
            session: ResolutionSession,
            status: number,
            body: Uint8Array
    ): ResolutionStep;

    transportFailed(
            session: ResolutionSession,
            kind: TransportFailureKind
    ): ResolutionStep;
}
