export function useResourceActions() {
    const copy = async (url: string) => {
        const parsed = requireHttps(url);
        await navigator.clipboard.writeText(parsed.toString());
    };

    const open = (url: string) => {
        const parsed = requireHttps(url);
        window.open(parsed.toString(), "_blank", "noopener,noreferrer");
    };

    const saveIfSupported = async (url: string): Promise<"saved" | "opened"> => {
        try {
            const parsed = requireHttps(url);
            const response = await fetch(parsed.toString(), { mode: "cors" });
            if (!response.ok) {
                throw new Error("save request failed");
            }
            const blob = await response.blob();
            const objectUrl = URL.createObjectURL(blob);
            const anchor = document.createElement("a");
            anchor.href = objectUrl;
            anchor.download = parsed.pathname.split("/").pop() || "resource";
            anchor.rel = "noopener noreferrer";
            anchor.click();
            URL.revokeObjectURL(objectUrl);
            return "saved";
        } catch {
            open(url);
            return "opened";
        }
    };

    return { copy, open, saveIfSupported };
}

function requireHttps(url: string) {
    const parsed = new URL(url);
    if (parsed.protocol !== "https:") {
        throw new Error("unsupported resource protocol");
    }
    return parsed;
}
