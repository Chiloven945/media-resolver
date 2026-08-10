#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi

export PATH="$HOME/.cargo/bin:$PATH"
rustup target add wasm32-unknown-unknown

rm -rf "$ROOT_DIR/worker/pkg"
(
  cd "$ROOT_DIR/crates/wasm"
  npx --yes wasm-pack@0.15.0 build \
    --target bundler \
    --release \
    --out-dir ../../worker/pkg \
    --out-name engine
)

node "$ROOT_DIR/scripts/patch-worker-wasm.mjs" "$ROOT_DIR/worker/pkg/engine.js"

(
  cd "$ROOT_DIR/web"
  bun install --frozen-lockfile
  cd "$ROOT_DIR/crates/wasm"
  npx --yes wasm-pack@0.15.0 build \
    --target web \
    --release \
    --out-dir ../../web/public/wasm \
    --out-name engine
  cd "$ROOT_DIR/web"
  export NUXT_PUBLIC_RESOLVER_ENDPOINT="${NUXT_PUBLIC_RESOLVER_ENDPOINT:-same-origin}"
  bun run generate
)
