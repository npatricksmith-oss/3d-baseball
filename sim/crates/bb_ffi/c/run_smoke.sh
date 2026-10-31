#!/bin/bash
# Build the static library and link the C smoke test against it (uses $CC or the zig shim).
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/../../.."
cargo build -q --release -p bb_ffi
CC=${CC:-$HOME/.local/bin/zigcc}
OUT=target/release/bb_ffi_smoke
"$CC" -O2 crates/bb_ffi/c/smoke.c target/release/libbb_ffi.a -lunwind -lm -lpthread -ldl -o "$OUT"
"$OUT"
