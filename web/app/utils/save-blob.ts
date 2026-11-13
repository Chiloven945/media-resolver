export function saveBlob(blob: Blob, filename: string) {
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = sanitizeFilename(filename) || "resource";
    anchor.rel = "noopener noreferrer";
    anchor.style.display = "none";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
}

export function sanitizeFilename(value: string): string {
    return value
        .replace(/[\\/:*?"<>|\u0000-\u001F]/g, "-")
        .replace(/\s+/g, " ")
        .trim()
        .slice(0, 180);
}
