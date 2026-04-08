#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

BUILD_HOST="$(hostname -s 2>/dev/null || echo unknown)"
BUILD_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
BUILD_SHA="$(git -C "$PROJECT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"

echo "=== Building pages/ ==="
echo "  host:  $BUILD_HOST"
echo "  time:  $BUILD_TS"
echo "  sha:   $BUILD_SHA"
cd "$PROJECT_DIR"

mkdir -p pages
touch pages/.nojekyll
trunk build --release --public-url /ibm-390-rs/
rsync -a --delete --exclude='.nojekyll' dist/ pages/

echo "=== Done ==="
echo "Pages built in: $PROJECT_DIR/pages/"
echo "To deploy: git add pages/ && git commit && git push"
