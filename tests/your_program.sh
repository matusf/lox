#!/usr/bin/env bash
set -euo pipefail
exec "${CARGO_TARGET_DIR:-./target}/debug/codecrafters-shim" "$@"
