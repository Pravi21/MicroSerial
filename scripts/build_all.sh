#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BUILD_DIR="${ROOT_DIR}/build/core"

cmake -S "${ROOT_DIR}/core" -B "${BUILD_DIR}" -DCMAKE_BUILD_TYPE=Release
cmake --build "${BUILD_DIR}" --config Release

echo "[microserial] Building GUI (release)"
cargo build --manifest-path "${ROOT_DIR}/gui/Cargo.toml" --release --locked
