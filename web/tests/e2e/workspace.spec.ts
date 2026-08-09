import { expect, type Page, test, type TestInfo } from "@playwright/test";

const corsHeaders = { "access-control-allow-origin": "*" };

const engineModule = String.raw`
export default async function init() {}

function sourceKey(input) {
  const key = input.match(/status\/(\d+)/)?.[1]
  if (!key) throw { code: 'invalid_input' }
  return key
}

function makeRoutes(key, options) {
  const routes = [
    { routeKey: 'r0', url: 'https://primary.invalid/' + key }
  ]
  if (options?.gatewayEndpoint) {
    routes.push({ routeKey: 'r1', url: 'https://managed.invalid/v1/resources/' + key })
  }
  routes.push({ routeKey: 'r' + routes.length, url: 'https://legacy.invalid/' + key })
  return routes
}

function requestStep(session) {
  const route = session.routes[session.index]
  return {
    kind: 'request',
    session,
    sourceKey: session.key,
    normalizedInput: 'https://example.invalid/item/' + session.key,
    request: {
      routeKey: route.routeKey,
      url: route.url,
      method: 'GET',
      headers: [],
      retryPolicy: {
        maxRetries: 2,
        delaysMs: [10, 20],
        retryStatuses: [429, 500, 502, 503, 504]
      }
    }
  }
}

function finalFailure(session) {
  const failures = session.failures || []
  const priority = [
    ['remote_not_found', 'not_found'],
    ['no_resources', 'no_resources'],
    ['rate_limited', 'rate_limited'],
    ['remote_rejected', 'rejected'],
    ['remote_unavailable', 'unavailable'],
    ['network_unavailable', 'network']
  ]
  for (const [code, failure] of priority) {
    if (failures.includes(failure)) return { kind: 'failed', error: { code } }
  }
  return { kind: 'failed', error: { code: 'invalid_response' } }
}

function advance(session, failure) {
  const next = {
    ...session,
    index: session.index + 1,
    failures: [...(session.failures || []), failure]
  }
  if (next.index >= next.routes.length) return finalFailure(next)
  return requestStep(next)
}

function resultFor(key) {
  if (key === '7001') {
    return {
      schemaVersion: 1,
      sourceKey: key,
      resources: [{
        id: key + ':video',
        kind: 'video',
        preferredUrl: 'https://assets.invalid/' + key + '-720.mp4',
        previewUrl: 'https://assets.invalid/' + key + '-preview.jpg',
        width: 720,
        height: 1280,
        variants: [
          {
            url: 'https://assets.invalid/' + key + '-720.mp4',
            mimeType: 'video/mp4',
            container: 'mp4',
            codec: 'h264',
            bitrate: 2200000,
            sizeBytes: 14000000,
            width: 720,
            height: 1280
          },
          {
            url: 'https://assets.invalid/' + key + '-480.mp4',
            mimeType: 'video/mp4',
            container: 'mp4',
            codec: 'h264',
            bitrate: 850000,
            sizeBytes: 6000000,
            width: 480,
            height: 854
          }
        ]
      }]
    }
  }

  if (key === '7002') {
    return {
      schemaVersion: 1,
      sourceKey: key,
      resources: [1, 2].map(index => ({
        id: key + ':' + index,
        kind: 'image',
        preferredUrl: 'https://assets.invalid/' + key + '-' + index + '.jpg',
        previewUrl: 'https://assets.invalid/' + key + '-' + index + '.jpg',
        width: 1200,
        height: 800,
        variants: [{
          url: 'https://assets.invalid/' + key + '-' + index + '.jpg',
          mimeType: 'image/jpeg',
          container: 'jpg',
          width: 1200,
          height: 800
        }]
      }))
    }
  }

  if (key === '7003') {
    return {
      schemaVersion: 1,
      sourceKey: key,
      resources: Array.from({ length: 12 }, (_, offset) => {
        const index = offset + 1
        return {
          id: key + ':' + index,
          kind: 'image',
          preferredUrl: 'https://assets.invalid/' + key + '-' + index + '.jpg',
          previewUrl: 'https://assets.invalid/' + key + '-' + index + '.jpg',
          width: 1200,
          height: 800,
          variants: [{
            url: 'https://assets.invalid/' + key + '-' + index + '.jpg',
            mimeType: 'image/jpeg',
            container: 'jpg',
            width: 1200,
            height: 800
          }]
        }
      })
    }
  }

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
        mimeType: 'image/jpeg',
        container: 'jpg',
        width: 1200,
        height: 800
      }]
    }]
  }
}

export function start(input, options) {
  const key = sourceKey(input)
  return requestStep({ key, index: 0, routes: makeRoutes(key, options), failures: [] })
}

export function respond(session, status, body) {
  let payload = {}
  try {
    payload = JSON.parse(new TextDecoder().decode(body || new Uint8Array()))
  } catch {}

  if (payload.outcome === 'restricted') {
    return { kind: 'failed', error: { code: 'remote_restricted' } }
  }
  if (payload.outcome === 'not_found') return advance(session, 'not_found')
  if (payload.outcome === 'no_resources') return advance(session, 'no_resources')
  if (payload.outcome === 'fallback') return advance(session, 'unavailable')

  if (status === 404) return advance(session, 'not_found')
  if (status === 429) return advance(session, 'rate_limited')
  if (status >= 500) return advance(session, 'unavailable')
  if (status >= 400) return advance(session, 'rejected')

  return { kind: 'resolved', result: resultFor(session.key) }
}

export function transport_failed(session) {
  return advance(session, 'network')
}
`;

async function setup(page: Page) {
    await page.route("**/wasm/engine.js*", route => route.fulfill({
        status: 200,
        contentType: "text/javascript",
        headers: { "cache-control": "no-store" },
        body: engineModule
    }));
    for (const pattern of [
        "https://primary.invalid/**",
        "https://managed.invalid/**",
        "https://legacy.invalid/**"
    ]) {
        await page.route(pattern, route => route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        }));
    }
    await page.route("https://assets.invalid/**", route => {
        const url = route.request().url();
        const isVideo = url.endsWith(".mp4");
        return route.fulfill({
            status: 200,
            contentType: isVideo
                    ? "video/mp4"
                    : "image/jpeg",
            headers: {
                "access-control-allow-origin": "*",
                "content-length": isVideo
                        ? "16"
                        : "12"
            },
            body: isVideo
                    ? "mock-video-bytes"
                    : "mock-image!!"
        });
    });
}

async function addTask(page: Page, id: string) {
    const input = page.getByLabel("Supported link").first();
    await expect(input).toBeEnabled();
    await input.fill(`https://example.test/user/status/${id}`);
    await input.press("Enter");
    await expect(input).toHaveValue("");
}

function desktopOnly(testInfo: TestInfo) {
    test.skip(testInfo.project.name !== "desktop");
}

test.beforeEach(async ({ page }) => {
    await setup(page);
    await page.goto("/", { waitUntil: "domcontentloaded" });
});

test(
        "desktop uses one global header and a constrained task sidebar",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await expect(page.getByTestId("app-header")).toBeVisible({ timeout: 15_000 });
            await expect(page.getByTestId("app-header")).toHaveCount(1);
            await expect(page.locator(".desktop-task-panel")).toBeVisible();
            const box = await page.locator(".desktop-task-panel").boundingBox();
            expect(box).not.toBeNull();
            expect(box!.width).toBeGreaterThanOrEqual(279);
            expect(box!.width).toBeLessThanOrEqual(381);
        }
);

test(
        "result header starts below the global application header",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "1000");

            const appHeader = await page.getByTestId("app-header").boundingBox();
            const resultHeader = await page.getByTestId("result-header").boundingBox();

            expect(appHeader).not.toBeNull();
            expect(resultHeader).not.toBeNull();
            expect(resultHeader!.y).toBeGreaterThanOrEqual(
                    appHeader!.y + appHeader!.height - 1
            );
        }
);

test("primary route resolves one task", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "1001");
    await expect(page.getByText(/1 resource ·/)).toBeVisible();
    await expect(page.getByText("Task 1").first()).toBeVisible();
});

test("creates, selects, reruns and removes one task", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "1101");
    await expect(page.getByText(/1 resource ·/)).toBeVisible();

    await page.getByLabel("Task actions").click();
    await page.getByText("Run again").click();
    await expect(page.getByText("Ready").first()).toBeVisible();

    await page.getByLabel("Task actions").click();
    await page.getByText("Remove").click();
    await expect(page.getByText("No task selected")).toBeVisible();
});

test("supports multiple tasks, selection and duplicate detection", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "2001");
    await addTask(page, "2002");
    await expect(page.getByText("Task 2").first()).toBeVisible();

    await page.locator("[data-task-sequence=\"1\"] button").first().click();
    await expect(page.getByRole("heading", { name: "Task 1" })).toBeVisible();

    await addTask(page, "2001");
    await expect(page.getByText("Task already exists")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Task 1" })).toBeVisible();
});

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
    await page.unroute("https://primary.invalid/**");
    let active = 0;
    let maximumActive = 0;
    await page.route("https://primary.invalid/**", async route => {
        active += 1;
        maximumActive = Math.max(maximumActive, active);
        await new Promise(resolve => setTimeout(resolve, 350));
        active -= 1;
        await route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
    });

    for (const id of ["4101", "4102", "4103", "4104", "4105"]) {
        await addTask(page, id);
    }

    await expect(page.getByText("5 / 5 complete")).toBeVisible({ timeout: 10000 });
    expect(maximumActive).toBe(4);
});

test("removes a queued task before it starts", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://primary.invalid/**");
    await page.route("https://primary.invalid/**", async route => {
        await new Promise(resolve => setTimeout(resolve, 900));
        await route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
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

test("cancel during first route never starts a fallback route", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://primary.invalid/**");
    await page.unroute("https://managed.invalid/**");
    let managedRequests = 0;
    await page.route("https://primary.invalid/**", async route => {
        await new Promise(resolve => setTimeout(resolve, 1500));
        try {
            await route.fulfill({
                status: 200,
                contentType: "application/json",
                headers: corsHeaders,
                body: "{}"
            });
        } catch {
            // The page may abort the request first.
        }
    });
    await page.route("https://managed.invalid/**", route => {
        managedRequests += 1;
        return route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
    });

    await addTask(page, "4301");
    await expect(page.getByText("Connecting").first()).toBeVisible();
    await page.getByRole("button", { name: "Cancel" }).last().click();
    await expect(page.getByText("Task cancelled")).toBeVisible();
    await page.waitForTimeout(100);
    expect(managedRequests).toBe(0);
});

test("primary unavailable falls through to managed route", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://primary.invalid/**");
    let primaryRequests = 0;
    let managedRequests = 0;
    await page.route("https://primary.invalid/**", route => {
        primaryRequests += 1;
        return route.fulfill({
            status: 503,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
    });
    await page.unroute("https://managed.invalid/**");
    await page.route("https://managed.invalid/**", route => {
        managedRequests += 1;
        return route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
    });

    await addTask(page, "4401");
    await expect(page.getByText(/1 resource ·/)).toBeVisible();
    expect(primaryRequests).toBe(3);
    expect(managedRequests).toBe(1);
});

test("primary browser access failure falls through automatically", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://primary.invalid/**");
    let managedRequests = 0;
    await page.route("https://primary.invalid/**", route => route.abort("connectionfailed"));
    await page.unroute("https://managed.invalid/**");
    await page.route("https://managed.invalid/**", route => {
        managedRequests += 1;
        return route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
    });

    await addTask(page, "4501");
    await expect(page.getByText(/1 resource ·/)).toBeVisible({ timeout: 10000 });
    expect(managedRequests).toBe(1);
});

test("exhausted rate-limit retries advance to the next route", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://primary.invalid/**");
    let primaryRequests = 0;
    await page.route("https://primary.invalid/**", route => {
        primaryRequests += 1;
        return route.fulfill({
            status: 429,
            contentType: "application/json",
            headers: corsHeaders,
            body: "{}"
        });
    });

    await addTask(page, "4601");
    await expect(page.getByText(/1 resource ·/)).toBeVisible();
    expect(primaryRequests).toBe(3);
});

test(
        "primary unavailable payload can continue through the configured managed route",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await page.unroute("https://primary.invalid/**");
            let managedRequests = 0;
            await page.route("https://primary.invalid/**", route => route.fulfill({
                status: 200,
                contentType: "application/json",
                headers: corsHeaders,
                body: JSON.stringify({ outcome: "fallback" })
            }));
            await page.unroute("https://managed.invalid/**");
            await page.route("https://managed.invalid/**", route => {
                managedRequests += 1;
                return route.fulfill({
                    status: 200,
                    contentType: "application/json",
                    headers: corsHeaders,
                    body: "{}"
                });
            });

            await addTask(page, "4701");
            await expect(page.getByText(/1 resource ·/)).toBeVisible();
            expect(managedRequests).toBe(1);
        }
);

test(
        "restricted source is terminal and never starts anonymous fallbacks",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await page.unroute("https://primary.invalid/**");
            await page.unroute("https://managed.invalid/**");
            let managedRequests = 0;
            await page.route("https://primary.invalid/**", route => route.fulfill({
                status: 200,
                contentType: "application/json",
                headers: corsHeaders,
                body: JSON.stringify({ outcome: "restricted" })
            }));
            await page.route("https://managed.invalid/**", route => {
                managedRequests += 1;
                return route.fulfill({
                    status: 200,
                    contentType: "application/json",
                    headers: corsHeaders,
                    body: "{}"
                });
            });

            await addTask(page, "4801");
            await expect(page.getByText("Access unavailable")).toBeVisible();
            expect(managedRequests).toBe(0);
        }
);

test("all unavailable routes produce one final neutral failure", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    for (const pattern of [
        "https://primary.invalid/**",
        "https://managed.invalid/**",
        "https://legacy.invalid/**"
    ]) {
        await page.unroute(pattern);
        await page.route(pattern, route => route.fulfill({
            status: 200,
            contentType: "application/json",
            headers: corsHeaders,
            body: JSON.stringify({ outcome: "fallback" })
        }));
    }

    await addTask(page, "4901");
    await expect(page.getByText("This source isn't available through the current access methods."))
            .toBeVisible();
    await expect(page.getByText("Source unavailable")).toHaveCount(1);
});

test("advanced recovery uses the saved route session", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    for (const pattern of [
        "https://primary.invalid/**",
        "https://managed.invalid/**",
        "https://legacy.invalid/**"
    ]) {
        await page.unroute(pattern);
        await page.route(pattern, route => route.abort("connectionfailed"));
    }

    await addTask(page, "5001");
    await expect(page.getByText("Network unavailable")).toBeVisible({ timeout: 10000 });
    await page.getByRole("button", { name: "Advanced recovery" }).click();
    await page.getByLabel("Response text").fill("{}");
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(page.getByText(/1 resource ·/)).toBeVisible();
});

test("route health skips a recently blocked route for later tasks", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await page.unroute("https://primary.invalid/**");
    let primaryRequests = 0;
    await page.route("https://primary.invalid/**", route => {
        primaryRequests += 1;
        return route.abort("connectionfailed");
    });

    await addTask(page, "5100");
    await expect(page.getByText(/1 resource ·/)).toBeVisible({ timeout: 10000 });

    for (let index = 1; index < 20; index += 1) {
        await addTask(page, String(5100 + index));
    }
    await expect(page.getByText("20 / 20 complete")).toBeVisible({ timeout: 15000 });
    expect(primaryRequests).toBe(1);
});

test(
        "surfaces engine initialization failure without implementation details",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await page.unroute("**/wasm/engine.js*");
            await page.route("**/wasm/engine.js*", route => route.abort("failed"));
            await page.reload({ waitUntil: "domcontentloaded" });
            await expect(page.getByText("Initialization failed")).toBeVisible({ timeout: 15_000 });
            await expect(page.getByLabel("Supported link")).toBeDisabled();
        }
);

test("mobile task drawer creates a task and closes on selection", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "mobile");
    await page.getByLabel("Open tasks").click();
    await expect(page.getByText("Manage active and completed tasks.")).toBeVisible();

    const drawer = page.getByRole("dialog");
    const input = drawer.getByLabel("Supported link");
    await expect(input).toBeEnabled();
    await input.fill("https://example.test/user/status/6001");
    await input.press("Enter");
    await expect(input).toHaveValue("");
    await expect(page.getByText("Manage active and completed tasks.")).not.toBeVisible();
    await expect(page.getByRole("heading", { name: "Task 1" })).toBeVisible();

    await page.getByLabel("Open tasks").click();
    await page.getByRole("dialog").locator("[data-task-sequence=\"1\"] button").first().click();
    await expect(page.getByText("Manage active and completed tasks.")).not.toBeVisible();
});

test(
        "single resource uses the detail workspace instead of a card grid",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "7001");
            const detail = page.getByTestId("resource-detail");
            await expect(detail).toBeVisible();
            await expect(page.getByTestId("resource-grid")).toHaveCount(0);
            await expect(page.getByText("Video", { exact: true })).toBeVisible();

            const detailBox = await detail.boundingBox();
            const viewerBox = await page.getByTestId("resource-viewer").first().boundingBox();
            expect(detailBox).not.toBeNull();
            expect(viewerBox).not.toBeNull();
            expect(detailBox!.width).toBeGreaterThan(700);
            expect(viewerBox!.width).toBeGreaterThan(420);

            const viewport = page.viewportSize();
            expect(viewport).not.toBeNull();
            expect(viewerBox!.height).toBeLessThanOrEqual(
                    Math.min(viewport!.height * 0.62, 42 * 16) + 2
            );
        }
);

test("multiple resources use the resource grid", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "7002");
    await expect(page.getByTestId("resource-grid")).toBeVisible();
    await expect(page.getByTestId("resource-card")).toHaveCount(2);
    await expect(page.getByTestId("resource-detail")).toHaveCount(0);
});

test(
        "multiple resources can scroll all the way to the final card",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "7003");

            const scrollArea = page.getByTestId("result-scroll-area");
            const cards = page.getByTestId("resource-card");
            await expect(cards).toHaveCount(12);

            const before = await scrollArea.evaluate(element => (
                    {
                        clientHeight: element.clientHeight,
                        scrollHeight: element.scrollHeight
                    }
            ));
            expect(before.scrollHeight).toBeGreaterThan(before.clientHeight);

            await scrollArea.evaluate(element => {
                element.scrollTop = element.scrollHeight;
            });

            await expect.poll(async () => scrollArea.evaluate(element =>
                    element.scrollTop + element.clientHeight >= element.scrollHeight - 2
            )).toBe(true);

            const areaBox = await scrollArea.boundingBox();
            const lastCardBox = await cards.last().boundingBox();
            expect(areaBox).not.toBeNull();
            expect(lastCardBox).not.toBeNull();
            expect(lastCardBox!.y + lastCardBox!.height)
                    .toBeLessThanOrEqual(areaBox!.y + areaBox!.height + 2);
        }
);

test(
        "portrait media keeps its real aspect ratio and is not forced to 16:10",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "7001");
            const viewer = page.getByTestId("resource-viewer").first();
            await expect(viewer).toHaveAttribute("data-aspect-ratio", "720 / 1280");
            await expect(viewer).not.toHaveCSS("aspect-ratio", "8 / 5");
        }
);

test(
        "download is the primary resource action and resource copy actions are absent",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "7001");
            await expect(page.getByRole("button", { name: "Download", exact: true })).toBeVisible();
            await expect(page.getByText("Copy", { exact: true })).toHaveCount(0);
            await expect(page.getByText("Copy address", { exact: true })).toHaveCount(0);
        }
);

test("direct resource download creates a browser download", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "7101");

    const event = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download", exact: true }).click();
    const download = await event;
    expect(download.suggestedFilename()).toBe("resource-1.jpg");
    await expect(page.getByText("Resource downloaded")).toBeVisible();
});

test(
        "direct download failure falls back to the configured managed download route",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "7102");

            await page.unroute("https://assets.invalid/**");
            await page.route("https://assets.invalid/**", route => route.abort("connectionfailed"));
            let gatewayDownloads = 0;
            await page.route("https://managed.invalid/v1/download/**", route => {
                gatewayDownloads += 1;
                return route.fulfill({
                    status: 200,
                    contentType: "image/jpeg",
                    headers: {
                        "access-control-allow-origin": "*",
                        "access-control-expose-headers": "content-disposition, content-length",
                        "content-disposition": "attachment; filename=\"gateway-file.jpg\"",
                        "content-length": "13"
                    },
                    body: "gateway-image"
                });
            });

            const event = page.waitForEvent("download");
            await page.getByRole("button", { name: "Download", exact: true }).click();
            const download = await event;
            expect(download.suggestedFilename()).toBe("gateway-file.jpg");
            expect(gatewayDownloads).toBe(1);
        }
);

test(
        "download failure exposes the open-resource fallback without restoring copy",
        async ({ page }, testInfo) => {
            desktopOnly(testInfo);
            await addTask(page, "7103");

            await page.unroute("https://assets.invalid/**");
            await page.route("https://assets.invalid/**", route => route.abort("connectionfailed"));
            await page.route(
                    "https://managed.invalid/v1/download/**",
                    route => route.abort("connectionfailed")
            );

            await page.getByRole("button", { name: "Download", exact: true }).click();
            await expect(page.getByText("Download unavailable").first()).toBeVisible();
            await expect(page.getByRole("button", { name: "Open resource" })).toBeVisible();
            await expect(page.getByText("Copy", { exact: true })).toHaveCount(0);
        }
);

test("variant download uses the selected representation", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "7001");

    const event = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download 480×854" }).click();
    const download = await event;
    expect(download.suggestedFilename()).toBe("resource-1-480x854.mp4");
});

test("download shows an in-progress state", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    await addTask(page, "7001");

    await page.unroute("https://assets.invalid/**");
    await page.route("https://assets.invalid/**", async route => {
        if (route.request().url().endsWith("-720.mp4")) {
            await new Promise(resolve => setTimeout(resolve, 500));
        }
        return route.fulfill({
            status: 200,
            contentType: route.request().url().endsWith(".mp4")
                    ? "video/mp4"
                    : "image/jpeg",
            headers: { "access-control-allow-origin": "*", "content-length": "16" },
            body: "mock-video-bytes"
        });
    });

    const event = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download", exact: true }).click();
    await expect(page.getByRole("button", { name: /Preparing|Downloading/ })).toBeVisible();
    await event;
});

test("color mode control switches the semantic theme", async ({ page }, testInfo) => {
    desktopOnly(testInfo);
    const button = page.getByRole("button", { name: "Toggle color mode" });
    await expect(button).toBeVisible();
    const before = await page.evaluate(() => document.documentElement.className);
    await button.click();
    await expect.poll(() => page.evaluate(() => document.documentElement.className))
            .not
            .toBe(before);
});

test("mobile detail keeps download as the primary action", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "mobile");
    await page.getByLabel("Open tasks").click();
    const drawer = page.getByRole("dialog");
    const input = drawer.getByLabel("Supported link");
    await expect(input).toBeEnabled();
    await input.fill("https://example.test/user/status/7001");
    await input.press("Enter");
    await expect(input).toHaveValue("");

    await expect(page.getByTestId("resource-detail")).toBeVisible();
    await expect(page.getByRole("button", { name: "Download", exact: true })).toBeVisible();
    await expect(page.getByText("Copy", { exact: true })).toHaveCount(0);
});

