#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ARGS=("$@")
WIN_ARGS=()
for ((i=0; i<${#ARGS[@]}; i++)); do
  case "${ARGS[i]}" in
    --no-verify)
      WIN_ARGS+=("-NoVerify")
      ;;
    --out-dir)
      if [[ $((i+1)) -lt ${#ARGS[@]} ]]; then
        WIN_ARGS+=("-OutDir" "${ARGS[i+1]}")
        i=$((i+1))
      fi
      ;;
  esac
done

echo "[1/2] Package Linux/WSL artifact"
bash "$ROOT/scripts/package.sh" "$@"

if ! command -v powershell.exe >/dev/null 2>&1; then
  echo "powershell.exe not found; skipping Windows packaging" >&2
  exit 0
fi

echo "[2/2] Package Windows artifact"
WIN_ROOT="$(wslpath -w "$ROOT")"
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$WIN_ROOT\\scripts\\package.ps1" "${WIN_ARGS[@]}"
