#!/usr/bin/env bash
# Builds the browser version of Between into web/dist/.
# Usage: web/build.sh [OUT_DIR]   (OUT_DIR defaults to web/dist)
# Needs: wasm32-unknown-unknown target, slangc, and wasm-bindgen-cli matching
# the wasm-bindgen version in web/Cargo.lock. wasm-opt (binaryen) is optional.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="${1:-$here/dist}"

# Keep this machine's home directory out of the compiled wasm (it would
# otherwise embed paths like /home/you/.cargo/... in panic messages).
remap="--remap-path-prefix=$HOME=~"
[[ " ${RUSTFLAGS:-} " == *" $remap "* ]] || export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }$remap"

cargo build --manifest-path "$here/Cargo.toml" --release --target wasm32-unknown-unknown

mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" \
  "$here/target/wasm32-unknown-unknown/release/between_web.wasm"

if command -v wasm-opt >/dev/null; then
  wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int \
    -o "$out/between_web_bg.wasm" "$out/between_web_bg.wasm"
fi

cp "$here/www/index.html" "$out/index.html"
echo "built: $out ($(du -h "$out/between_web_bg.wasm" | cut -f1) wasm)"
