# media-resolver

A Rust-powered resolver for supported links, available as a native CLI and a browser application.

Version **0.1.0** uses a Rust-owned multi-route resolution engine. Input validation, route ordering,
fallback policy, response classification, and resource normalization all live in the shared core.
The CLI and browser are transport executors: they perform the requested HTTP operation and return
the outcome to Rust. The browser build remains a static Nuxt application with a thin WebAssembly
bridge.

## Features

- Native Rust CLI for one or many links
- Shared Rust resolution state machine with source-neutral public models
- Multiple automatic resolution routes with deterministic fallback policy
- Optional managed gateway configured at deployment time
- Thin WebAssembly bridge with no browser networking inside WASM
- Nuxt 4 + Nuxt UI 4 task workspace with a compact sidebar and resource-focused detail view
- Concurrent task queue with cancellation, route-aware retries, deduplication, and memory caches
- Route-health memory that avoids repeatedly using a temporarily blocked route
- Image, video, animation, preferred-resource, and full variant normalization
- Direct video variants plus stream representations with container/codec/bitrate/size metadata
- Advanced manual recovery only when all automatic browser routes are exhausted
- Direct browser downloads with optional managed download fallback
- Responsive desktop and mobile layouts
- System-aware light and dark themes
- Static deployment with no application server requirement
- No embedded credentials, API keys, analytics, or open proxy

## Repository layout

```text
crates/core   Source inspection, resolution state machine, route adapters, normalization
crates/cli    Native HTTP transport, concurrency, and terminal/JSON output
crates/wasm   Thin wasm-bindgen state-machine bridge
web           Nuxt UI single-page application and browser transport executor
tests         Route compatibility fixtures used by Rust tests
docs          Maintainer-facing protocol documentation
.github       CI, static deployment, and optional route smoke checks
```

Source-specific implementation details are confined to protocol source/route/schema modules,
protocol fixtures, and maintainer documentation. Public Rust models, WASM exports, TypeScript
interfaces, Vue components, CLI copy, README product copy, and user-facing application text use
neutral resource terminology.

## Requirements

- Rust stable with the `wasm32-unknown-unknown` target
- `wasm-pack`
- Bun 1.3.14 or newer within the 1.3 line

## Native CLI

Build it with:

```bash
cargo build --release -p media-resolver-cli
```

Resolve one link:

```bash
./target/release/media-resolver '<supported-link>'
```

Resolve several links concurrently:

```bash
./target/release/media-resolver -j 4 '<supported-link-1>' '<supported-link-2>'
```

Read links from standard input:

```bash
cat links.txt | ./target/release/media-resolver - --json --pretty
```

Key options:

```text
-j, --jobs <N>       concurrent jobs (default: 4)
    --json           emit JSON
    --pretty         pretty-print JSON
    --all-variants   include every resource variant
    --timeout <SEC>  request timeout (default: 15)
    --no-color       disable ANSI output
    --verbose        emit neutral diagnostic progress to stderr
```

Standard output is reserved for result data. Diagnostics and human-readable failures go to standard
error. The optional environment variable `MEDIA_RESOLVER_GATEWAY_ENDPOINT` enables the managed route
for native runs.

## Resolution model

The core exposes a small state-machine API:

```text
start_resolution
      ↓
Request ── transport ──► accept_response / accept_transport_failure
  ▲                                      │
  └────────────── next route ◄───────────┤
                                         ├─► Resolved
                                         └─► Failed
```

A request includes an opaque route key and a retry policy. Callers must not derive business behavior
from the route key. Explicit restricted-access outcomes are terminal; ordinary route unavailability,
rate exhaustion, malformed route responses, and transport failures can advance to another route.
Final errors are selected from the complete session failure history rather than whichever route
happened to run last.

Serialized resolution sessions contain only control state. They do not contain raw response bodies,
credentials, cookies, or resolved media payloads.

## Web application

Install dependencies and build the WASM module:

```bash
cd web
bun install
bun run wasm:build
bun run dev
```

The browser architecture is deliberately split:

```text
Nuxt task queue
      ↓
generic resolver executor
      ↓
Rust state machine ──► PreparedRequest
      ▲                    │
      │                    ▼
      └──── result ◄── browser transport
```

WASM does not perform HTTP. Browser CORS/network failures are converted into transport failures and
returned to the Rust engine, which decides whether another route should be attempted. A task only
enters the failed state after the resolution session itself becomes terminal.

### Optional managed service

A deployment can add a managed route without changing or rebuilding the Rust engine API:

```bash
NUXT_PUBLIC_RESOLVER_ENDPOINT=https://resolver.example.dev
```

The endpoint is public configuration, not a secret. The expected contracts are intentionally narrow:

```text
GET {endpoint}/v1/resources/{numeric-source-key}
GET {endpoint}/v1/download/{numeric-source-key}/{resource-id}
GET {endpoint}/v1/download/{numeric-source-key}/{resource-id}?variant={zero-based-index}
```

The resource endpoint must use the neutral `ResourceBundle` schema. The download endpoint resolves
the resource identifier server-side and returns the selected representation as an attachment;
callers never supply an arbitrary upstream URL, host, method, or header. The configured endpoint
must be HTTPS and may not contain user-info, query parameters, or fragments. It is not an arbitrary
URL proxy.

No token or API credential belongs in `NUXT_PUBLIC_RESOLVER_ENDPOINT`, the Nuxt bundle, or WASM.
Secrets needed by a managed service must remain in that service's secret storage.

### Static generation

```bash
cd web
bun run wasm:build
bun run generate
```

The generated site is written to `web/.output/public` and can be served by a static host. Set
`NUXT_APP_BASE_URL` when the site is hosted below a path prefix.

## Browser task behavior

- Tasks are scheduled with a configurable concurrency of 1–8.
- Every active task owns an `AbortController` outside Nuxt serializable state.
- Result cache entries live in memory for five minutes.
- Route-health entries also live only in memory; access-blocked routes are temporarily skipped,
  short network outages receive a shorter cooldown, and exhausted rate limits respect the available
  retry delay when possible.
- Raw remote bodies are passed directly from transport into WASM and are never written to Vue task
  state or local storage.
- Manual response recovery keeps its opaque route session in module memory and is offered only when
  a browser route could be opened manually but could not be read automatically.

## Privacy and persistence

The browser application keeps task inputs, resolution state, resource addresses, cache entries, and
route health in memory only. It does not persist task history or raw remote responses. Local storage
is limited to interface preferences such as concurrency and preview behavior.

The application contains no account system, analytics SDK, embedded secret, open proxy, or
background history database.

## Resource actions

Version 0.1.0 uses a download-first resource workflow:

- Download
- Open externally

The browser first attempts a CORS-readable resource request and saves the resulting `Blob` with a
local filename. When direct reading is unavailable and a managed endpoint is configured, it retries
through the restricted `/v1/download/{source-key}/{resource-id}` contract. If neither path succeeds,
the interface keeps Open externally available as the explicit fallback.

Download progress is kept separately from resolver task state. Known content length produces
determinate progress; otherwise the interface uses an indeterminate progress indicator. Streaming
representations are kept as variants and can be saved as their underlying playlist representation,
but the built-in preview does not add a dedicated streaming-player dependency in 0.1.0.

## Testing

Rust checks:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

WASM state-machine parity:

```bash
cd crates/wasm
wasm-pack test --node
```

Web checks:

```bash
cd web
bun run typecheck
bun run test
bunx playwright install chromium
bun run test:e2e
bun run generate
```

Normal tests mock transport. The non-blocking protocol smoke workflow uses optional repository
variables for live route health checks and does not gate pull requests.

## Build principles

- Rust is the single source of truth for input validation, route selection, fallback, error
  classification, and normalization.
- CLI and browser transports never encode provider-specific fallback rules.
- WASM remains a serialization/FFI boundary, not a networking layer.
- Nuxt owns queue state, generic transport execution, cancellation, and presentation.
- Route keys are opaque outside the Rust core.
- Raw remote bodies never enter Vue task state.
- Resource URLs are validated as HTTPS before they are exposed to callers.
- User-facing errors are mapped from stable neutral error codes rather than raw remote messages.
- Source identifiers remain strings across Rust, WASM, and TypeScript.
- Explicit restricted access is terminal for anonymous routes.

## 0.1.0 scope

Included:

- supported canonical and alternate link forms
- multiple automatic resolution routes
- optional managed resolution and download gateway contracts
- image, video, and animation resources
- preferred selection across direct and streaming variants
- native CLI and JSON output
- WebAssembly state-machine bridge
- Nuxt UI task workspace
- multiple concurrent tasks
- cancellation, retry policy, route health, deduplication, and memory caches
- advanced manual recovery
- responsive desktop/mobile interface
- light/dark mode
- static deployment

Explicitly out of scope:

- accounts or user authorization
- required application server
- persistent task history
- arbitrary/open proxy endpoints
- ZIP export
- browser extension or PWA
- analytics
- localization framework

Maintainers can find route-specific compatibility notes in `docs/protocol.md`.

## License

Released into the public domain under the Unlicense. See `LICENSE`.
