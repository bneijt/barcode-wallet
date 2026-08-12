#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Serving Barcode Wallet at http://localhost:8080 (Ctrl+C to stop)"
exec trunk serve
