export function buildManagedDownloadUrl(
        endpoint: string,
        sourceKey: string,
        resourceId: string,
        variantIndex?: number
): string | undefined {
    const value = endpoint.trim();
    if (!value) {
        return undefined;
    }

    try {
        const url = new URL(value);
        if (url.protocol !== "https:" || url.username || url.password || url.search || url.hash) {
            return undefined;
        }
        const basePath = url.pathname.replace(/\/+$/, "");
        url.pathname =
                `${basePath}/v1/download/${encodeURIComponent(sourceKey)}/${encodeURIComponent(
                        resourceId)}`;
        if (variantIndex !== undefined && variantIndex >= 0) {
            url.searchParams.set("variant", String(variantIndex));
        }
        return url.toString();
    } catch {
        return undefined;
    }
}
