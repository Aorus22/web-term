#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
mkdir -p "${ROOT_DIR}/desktop-gpui/test-support"
(cd "${ROOT_DIR}/be" && go build -o "${ROOT_DIR}/desktop-gpui/test-support/backend" ./cmd/server)
