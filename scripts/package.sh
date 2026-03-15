#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/package.sh [--target <triple>] [--out-dir <dir>] [--no-verify]

Builds a Linux/WSL release artifact into dist/:
  chronicle-v<version>-<target>.tar.gz (+ .sha256)
EOF
}

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET=""
OUT_DIR="dist"
VERIFY=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET="${2:-}"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="${2:-}"
      shift 2
      ;;
    --no-verify)
      VERIFY=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

VERSION="$(python3 - <<'PY'
import tomllib
with open("Cargo.toml", "rb") as f:
  print(tomllib.load(f)["package"]["version"])
PY
)"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-wsl}"

if [[ "$VERIFY" -eq 1 ]]; then
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test --locked
fi

if [[ -n "$TARGET" ]]; then
  cargo build --release --locked --target "$TARGET"
  BIN="$ROOT/$CARGO_TARGET_DIR/$TARGET/release/chronicle"
else
  cargo build --release --locked
  BIN="$ROOT/$CARGO_TARGET_DIR/release/chronicle"
  TARGET="$(rustc -vV | sed -n 's/^host: //p')"
fi

if [[ ! -f "$BIN" ]]; then
  echo "Binary not found: $BIN" >&2
  exit 1
fi

chmod +x "$BIN" || true

DIST="$ROOT/$OUT_DIR"
STAGE="$DIST/stage"
PKG_NAME="chronicle-v$VERSION-$TARGET"
PKG_DIR="$STAGE/$PKG_NAME"

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/completions"

cp -f "$BIN" "$PKG_DIR/chronicle"
cp -f "$ROOT/README.md" "$PKG_DIR/README.md"

for shell in bash zsh fish powershell elvish; do
  ext="$shell"
  if [[ "$shell" == "powershell" ]]; then
    ext="ps1"
  fi
  "$BIN" completions "$shell" > "$PKG_DIR/completions/chronicle.$ext"
done

mkdir -p "$DIST"
ARCHIVE="$DIST/$PKG_NAME.tar.gz"
rm -f "$ARCHIVE" "$ARCHIVE.sha256"

tar -C "$STAGE" -czf "$ARCHIVE" "$PKG_NAME"
(
  cd "$DIST"
  sha256sum "$(basename "$ARCHIVE")" > "$(basename "$ARCHIVE").sha256"
)

echo "Wrote:"
echo "  $ARCHIVE"
echo "  $ARCHIVE.sha256"
