# media-resolver

A Rust-powered resolver for supported links, available as a native CLI and a browser application.

Version **0.1.0** is intentionally focused: a small Rust engine owns validation, protocol handling,
response parsing, and resource normalization; the CLI supplies native HTTP transport; the browser
build exposes the same engine through WebAssembly while Nuxt handles transport and task
orchestration.

## Features

- Native Rust CLI for one or many links
- Shared Rust core with a stable, source-neutral resource model
- Thin WebAssembly bridge with no browser networking inside WASM
- Nuxt 4 + Nuxt UI 4 task workspace
- Concurrent task queue with cancellation, retries, deduplication, and a short in-memory cache
- Image, video, animation, preferred-resource, and variant normalization
- Responsive desktop and mobile layouts
- System-aware light and dark themes
- Static deployment with no application server
- No embedded credentials, API keys, analytics, or third-party proxy

## Repository layout

```text
crates/core   Shared validation, protocol adapter, parsing, normalization
crates/cli    Native transport, concurrency, and terminal/JSON output
crates/wasm   Thin wasm-bindgen bridge
web           Nuxt UI single-page application
tests         Protocol fixtures used by Rust tests
.github       CI, static deployment, and optional protocol smoke checks
```

Source-specific implementation details are intentionally confined to the protocol adapter and
protocol compatibility fixtures/tests. Public Rust models, WASM exports, TypeScript interfaces, Vue
components, CLI copy, and user-facing application text use neutral resource terminology.

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
error.

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
Nuxt task queue -> browser fetch -> raw response -> WASM -> ResourceBundle
```

The WASM module does not perform HTTP requests. Browser restrictions therefore remain explicit and
cancellable through `AbortController`, while native transport remains independent.

Because the browser transport is direct, a remote source can restrict cross-origin access
independently of this application. When that happens, the task fails with a neutral browser-blocked
error; the project intentionally does not bypass that restriction with a server or public proxy.

### Static generation

```bash
cd web
bun run wasm:build
bun run generate
```

The generated site is written to `web/.output/public` and can be served by a static host. Set
`NUXT_APP_BASE_URL` when the site is hosted below a path prefix.

## Privacy and persistence

The browser application keeps task inputs and resolved resource addresses in memory only. It does
not persist task history or raw remote responses. Local storage is limited to interface preferences
such as concurrency and preview behavior.

The application contains no account system, server, analytics SDK, embedded secret, public proxy, or
background history database.

## Resource actions

Version 0.1.0 guarantees two resource actions in the browser:

- Open resource
- Copy address

The resource-action layer also contains a guarded best-effort save helper for future use, but the
0.1.0 interface intentionally exposes only the two actions above because direct browser retrieval
can be restricted by cross-origin policy. The application does not promise ZIP export or bulk
download in 0.1.0.

## Testing

Rust checks:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

WASM parity test:

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

The normal test suite mocks remote transport. A separate non-blocking workflow can use the
repository variable `SMOKE_LINK` for an optional real protocol health check without hard-coding a
source URL into CI.

## Build principles

- Rust is the single source of truth for input validation and response normalization.
- CLI and WASM do not duplicate resolver rules.
- WASM remains a serialization/FFI boundary, not a networking layer.
- Nuxt owns browser transport, queue state, retries, cancellation, and presentation.
- Raw remote bodies never enter Vue task state.
- Resource URLs are validated as HTTPS before they are exposed to callers.
- User-facing errors are mapped from stable error codes rather than displaying raw remote messages.
- Source identifiers remain strings across Rust, WASM, and TypeScript.

## 0.1.0 scope

Included:

- supported canonical and alternate link forms
- image, video, and animation resources
- preferred resource selection and all variants
- native CLI and JSON output
- WebAssembly bridge
- Nuxt UI task workspace
- multiple concurrent tasks
- cancellation, retries, deduplication, and memory cache
- responsive desktop/mobile interface
- light/dark mode
- static deployment

Explicitly out of scope:

- accounts or OAuth
- application server
- persistent task history
- ZIP export
- browser extension or PWA
- analytics
- public CORS/download proxy
- localization framework

## License

This project is licensed under the Unlicense license. See the [LICENSE](LICENSE) file for details.
