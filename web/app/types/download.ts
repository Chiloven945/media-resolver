export type DownloadState = "idle" | "preparing" | "downloading" | "completed" | "failed";

export interface ResourceDownload {
    state: DownloadState;
    progress?: number;
    fallbackUrl?: string;
}

export type DownloadResult = "downloaded" | "failed" | "cancelled";
