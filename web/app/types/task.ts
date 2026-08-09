import type { ResourceBundle } from "./engine";

export type TaskState =
        | "queued"
        | "connecting"
        | "processing"
        | "ready"
        | "failed"
        | "cancelled";

export interface TaskError {
    code: string;
}

export interface ResolveTask {
    id: string;
    sequence: number;
    input: string;
    sourceKey?: string;
    normalizedInput?: string;
    state: TaskState;
    createdAt: number;
    startedAt?: number;
    completedAt?: number;
    result?: ResourceBundle;
    error?: TaskError;
}

export interface PublicErrorMessage {
    title: string;
    description: string;
}

const PUBLIC_ERRORS: Record<string, PublicErrorMessage> = {
    invalid_input: {
        title: "Invalid link",
        description: "Enter a valid link."
    },
    unsupported_input: {
        title: "Unsupported link",
        description: "This link isn't supported."
    },
    remote_not_found: {
        title: "Source unavailable",
        description: "The source is no longer available."
    },
    remote_unavailable: {
        title: "Source unavailable",
        description: "This source isn't available through the current access methods."
    },
    remote_restricted: {
        title: "Access unavailable",
        description: "This source requires access that isn't available in the current session."
    },
    remote_rejected: {
        title: "Request rejected",
        description: "The source could not complete this request."
    },
    rate_limited: {
        title: "Too many requests",
        description: "Too many requests. Try again shortly."
    },
    invalid_response: {
        title: "Unexpected response",
        description: "The source returned an unexpected response."
    },
    no_resources: {
        title: "Nothing found",
        description: "No usable resources were found."
    },
    network_unavailable: {
        title: "Network unavailable",
        description: "The available sources couldn't be reached."
    },
    network_error: {
        title: "Network unavailable",
        description: "The available sources couldn't be reached."
    },
    internal: {
        title: "Unable to complete task",
        description: "An unexpected local error occurred."
    }
};

export function getPublicErrorMessage(code?: string): PublicErrorMessage {
    return PUBLIC_ERRORS[code || "internal"] || PUBLIC_ERRORS.internal!;
}
