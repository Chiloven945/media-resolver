import { describe, expect, it } from "vitest";
import { getPublicErrorMessage } from "../app/types/task";

describe("public error copy", () => {
    it("maps unavailable sources to neutral UI text", () => {
        expect(getPublicErrorMessage("remote_unavailable")).toEqual({
            title: "Source unavailable",
            description: "This source isn't available through the current access methods."
        });
    });

    it("distinguishes restricted access without exposing provider details", () => {
        expect(getPublicErrorMessage("remote_restricted")).toEqual({
            title: "Access unavailable",
            description: "This source requires access that isn't available in the current session."
        });
    });

    it("maps final transport exhaustion to a neutral network error", () => {
        expect(getPublicErrorMessage("network_unavailable").title).toBe("Network unavailable");
    });

    it("uses a safe fallback", () => {
        expect(getPublicErrorMessage("unknown").title).toBe("Unable to complete task");
    });
});
