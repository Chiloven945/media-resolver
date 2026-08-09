import { expect, type Page, test, type TestInfo } from "@playwright/test";

const engineModule = `
export default async function init() {}
export function prepare(input) {
  const key = input.match(/status\\/(\\d+)/)?.[1]
  if (!key) throw { code: 'invalid_input' }
  return {
    key,
    normalizedInput: 'https://example.invalid/item/' + key,
    request: { key, url: 'https://resolver.invalid/' + key, method: 'GET', headers: [] }
  }
}
export function complete(input, status) {
  if (status === 404) throw { code: 'remote_not_found' }
  if (status === 429) throw { code: 'rate_limited' }
  if (status >= 400) throw { code: 'remote_rejected' }
  const key = input.match(/status\\/(\\d+)/)?.[1]
  return {
    schemaVersion: 1,
    sourceKey: key,
    resources: [{
      id: key + ':1',
      kind: 'image',
      preferredUrl: 'https://assets.invalid/' + key + '.jpg',
      previewUrl: 'https://assets.invalid/' + key + '.jpg',
      width: 1200,
      height: 800,
      variants: [{
        url: 'https://assets.invalid/' + key + '.jpg',
        mimeType: 'image/jpeg', width: 1200, height: 800
      }]
    }]
  }
}
`;

async function setup(page: Page) {
    await page.route("**/wasm/engine.js", route => route.fulfill({
        status: 200,
        contentType: "text/javascript",
        headers: { "cache-control": "no-store" },
        body: engineModule
    }));
    await page.route("https://resolver.invalid/**", route => route.fulfill({
        status: 200,
        contentType: "application/json",
        body: "{}"
    }));
    await page.route("https://assets.invalid/**", route => route.fulfill({
        status: 200,
        contentType: "image/svg+xml",
        body: "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"800\" />"
    }));
}

async function addTask(page: Page, id: string) {
    const input = page.getByLabel("Supported link").first();
    await input.fill(`https://example.test/user/status/${id}`);
    await input.press("Enter");
    await expect(input).toHaveValue("");
}

function desktopOnly(testInfo: TestInfo) {
    test.skip(testInfo.project.name !== "desktop");
}

test.beforeEach(async ({ page }) => {
    await setup(page);
    await page.goto("/");
    await expect(page.getByText("Media Resolver").first()).toBeVisible();
});

test("creates, selects, reruns and removes one task", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "1001");
    await expect(page.getByText("1 resource found")).toBeVisible();
    await expect(page.getByText("Task 1").first()).toBeVisible();

    await page.getByLabel("Task actions").click();
    await page.getByText("Run again").click();
    await expect(page.getByText("Ready").first()).toBeVisible();

    await page.getByLabel("Task actions").click();
    await page.getByText("Remove").click();
    await expect(page.getByText("No active tasks")).toBeVisible();
});

test(
        "supports multiple tasks, explicit selection and duplicate detection",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "2001");
            await addTask(page, "2002");
            await expect(page.getByText("Task 2").first()).toBeVisible();

            await page.locator("[data-task-sequence=\"1\"] button").first().click();
            await expect(page.getByRole("heading", { name: "Task 1" })).toBeVisible();

            await addTask(page, "2001");
            await expect(page.getByText("Task already exists")).toBeVisible();
            await expect(page.getByRole("heading", { name: "Task 1" })).toBeVisible();
        }
);

test("batch add creates multiple tasks", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.getByRole("button", { name: "Batch add" }).click();
    await page.getByLabel("Links").fill([
        "https://example.test/user/status/3001",
        "https://example.test/user/status/3002",
        "https://example.test/user/status/3003"
    ].join("\n"));
    await page.getByRole("button", { name: /Add 3 tasks/ }).click();
    await expect(page.getByText("Task 3").first()).toBeVisible();
});

test("limits concurrent remote work to the configured queue size", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://resolver.invalid/**");
    let active = 0;
    let maximumActive = 0;
    await page.route("https://resolver.invalid/**", async (route) => {
        active += 1;
        maximumActive = Math.max(maximumActive, active);
        await new Promise(resolve => setTimeout(resolve, 350));
        active -= 1;
        await route.fulfill({ status: 200, contentType: "application/json", body: "{}" });
    });

    for (const id of ["4101", "4102", "4103", "4104", "4105"]) {
        await addTask(page, id);
    }

    await expect(page.getByText("5 / 5 completed")).toBeVisible({ timeout: 10000 });
    expect(maximumActive).toBe(4);
});

test("removes a queued task before it starts", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://resolver.invalid/**");
    await page.route("https://resolver.invalid/**", async (route) => {
        await new Promise(resolve => setTimeout(resolve, 900));
        await route.fulfill({ status: 200, contentType: "application/json", body: "{}" });
    });

    for (const id of ["4201", "4202", "4203", "4204", "4205"]) {
        await addTask(page, id);
    }

    const fifth = page.locator("[data-task-sequence=\"5\"]");
    await expect(fifth.getByText("Queued")).toBeVisible();
    await fifth.getByLabel("Task actions").click();
    await page.getByText("Remove").click();
    await expect(page.locator("[data-task-sequence=\"5\"]")).toHaveCount(0);
});

test("cancels an active request", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://resolver.invalid/**");
    await page.route("https://resolver.invalid/**", async (route) => {
        await new Promise(resolve => setTimeout(resolve, 1500));
        try {
            await route.fulfill({ status: 200, contentType: "application/json", body: "{}" });
        } catch {
            // The request may already have been cancelled by the page.
        }
    });

    await addTask(page, "4301");
    await expect(page.getByText("Connecting").first()).toBeVisible();
    await page.getByRole("button", { name: "Cancel" }).last().click();
    await expect(page.getByText("Task cancelled")).toBeVisible();
});

test("retries a failed task manually", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://resolver.invalid/**");
    let requests = 0;
    await page.route("https://resolver.invalid/**", async (route) => {
        requests += 1;
        await route.fulfill({
            status: requests === 1
                    ? 404
                    : 200,
            contentType: "application/json",
            body: "{}"
        });
    });

    await addTask(page, "4401");
    await expect(page.getByText("Source unavailable")).toBeVisible();
    await page.getByRole("button", { name: "Retry" }).last().click();
    await expect(page.getByText("1 resource found")).toBeVisible();
    expect(requests).toBe(2);
});

test("offers local continuation when direct browser access is blocked", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://resolver.invalid/**");
    let requests = 0;
    await page.route("https://resolver.invalid/**", route => {
        requests += 1;
        return route.abort("connectionfailed");
    });

    await addTask(page, "4501");
    await expect(page.getByText("Direct access blocked"))
            .toBeVisible({ timeout: 10000 });
    expect(requests).toBe(1);

    await page.getByRole("button", { name: "Continue from response" }).click();
    await page.getByLabel("Response text").fill("{}");
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(page.getByText("1 resource found")).toBeVisible();
});

test(
        "surfaces engine initialization failure without implementation details",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await page.unroute("**/wasm/engine.js");
            await page.route("**/wasm/engine.js", route => route.abort("failed"));
            await page.reload();
            await expect(page.getByText("Initialization failed")).toBeVisible();
            await expect(page.getByLabel("Supported link")).toBeDisabled();
        }
);

test("mobile task drawer creates a task and closes on selection", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "mobile");
    await page.getByLabel("Open tasks").click();
    await expect(page.getByText("Manage active and completed tasks.")).toBeVisible();

    const drawer = page.getByRole("dialog");
    const input = drawer.getByLabel("Supported link");
    await input.fill("https://example.test/user/status/5001");
    await input.press("Enter");
    await expect(page.getByText("Manage active and completed tasks.")).not.toBeVisible();
    await expect(page.getByRole("heading", { name: "Task 1" })).toBeVisible();

    await page.getByLabel("Open tasks").click();
    await page.getByRole("dialog").locator("[data-task-sequence=\"1\"] button").first().click();
    await expect(page.getByText("Manage active and completed tasks.")).not.toBeVisible();
});
