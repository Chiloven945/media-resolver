import { describe, expect, it } from "vitest";
import { getPublicErrorMessage } from "../app/types/task";

describe("public error copy", () => {
    it("maps remote errors to neutral UI text", () => {
        expect(getPublicErrorMessage("remote_not_found")).toEqual({
            title: "Source unavailable",
            description: "The source is no longer available."
        });
    });


    it("explains the browser-only fallback without exposing implementation details", () => {
        expect(getPublicErrorMessage("browser_blocked")).toEqual({
            title: "Direct access blocked",
            description: "Your browser cannot read this response directly. Open the response in a new tab, copy it, then continue here."
        });
    });

    it("uses a safe fallback", () => {
        expect(getPublicErrorMessage("unknown").title).toBe("Unable to complete task");
    });
});
