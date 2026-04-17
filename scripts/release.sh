#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

usage() {
  cat <<'EOF'
Usage:
  scripts/release.sh build
  scripts/release.sh publish
  scripts/release.sh release

Environment:
  PYPI_TOKEN   Required for `publish` and `release`.

Notes:
  - Builds only the minimal release set:
    - linux x86_64
    - windows x86_64
    - CPython 3.11, 3.12, 3.13, 3.14
  - Requires `uv` and `maturin` to be available.
  - Windows wheels can only be built on Windows.
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1" >&2
    exit 1
  }
}

build_linux() {
  for py in 3.11 3.12 3.13 3.14; do
    uv run --with "maturin>=1.13,<2.0" maturin build \
      --release \
      --out "${DIST_DIR}" \
      --interpreter "python${py}" \
      --target x86_64-unknown-linux-gnu
  done
}

build_windows() {
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      for py in 3.11 3.12 3.13 3.14; do
        uv run --with "maturin>=1.13,<2.0" maturin build \
          --release \
          --out "${DIST_DIR}" \
          --interpreter "python${py}" \
          --target x86_64-pc-windows-msvc
      done
      ;;
    *)
      echo "Skipping Windows wheel build on non-Windows host." >&2
      ;;
  esac
}

build_sdist() {
  uv run --with "maturin>=1.13,<2.0" maturin sdist --out "${DIST_DIR}"
}

build_all() {
  rm -rf "${DIST_DIR}"
  mkdir -p "${DIST_DIR}"
  build_linux
  build_windows
  build_sdist
}

publish_all() {
  : "${PYPI_TOKEN:?PYPI_TOKEN must be set}"
  uv publish "${DIST_DIR}"/* --token "${PYPI_TOKEN}"
}

main() {
  require_cmd uv

  case "${1:-}" in
    build)
      build_all
      ;;
    publish)
      publish_all
      ;;
    release)
      build_all
      publish_all
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main "$@"
