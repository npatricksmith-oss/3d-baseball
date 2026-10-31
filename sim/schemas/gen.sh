#!/bin/bash
# Regenerate Rust code from the .fbs schemas. flatc lives in the fedora-toolbox-45 toolbx
# container (the host has no flatc); generated code is committed so builds never need flatc.
set -euo pipefail
cd "$(dirname "$0")"
rm -rf ../crates/bb_schema/src/generated
mkdir -p ../crates/bb_schema/src/generated
run() { if command -v flatc >/dev/null; then "$@"; else toolbox run -c fedora-toolbox-45 "$@"; fi; }
run flatc --rust --gen-all --rust-module-root-file -o ../crates/bb_schema/src/generated all.fbs
