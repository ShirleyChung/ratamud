#!/usr/bin/env bash
set -euo pipefail

# Resume Codex in this repository.
# Default: resume the most recent Codex session for this project.
#
# Usage:
#   scripts/resume_codex.sh
#   scripts/resume_codex.sh --picker
#   scripts/resume_codex.sh --last "continue building the Rust project"
#   scripts/resume_codex.sh <session-id> "continue from here"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CODEX_BIN="${CODEX_BIN:-codex}"

cd "$REPO_DIR"

if ! command -v "$CODEX_BIN" >/dev/null 2>&1; then
  echo "codex CLI not found."
  echo "Install or expose it in PATH, or run with CODEX_BIN=/path/to/codex scripts/resume_codex.sh"
  exit 127
fi

echo "Repository: $REPO_DIR"
echo "Codex: $(command -v "$CODEX_BIN")"

if [[ "${1:-}" == "--picker" ]]; then
  shift
  exec "$CODEX_BIN" resume -C "$REPO_DIR" "$@"
fi

if [[ "${1:-}" == "--last" ]]; then
  shift
  exec "$CODEX_BIN" resume --last -C "$REPO_DIR" "$@"
fi

if [[ "$#" -gt 0 ]]; then
  exec "$CODEX_BIN" resume -C "$REPO_DIR" "$@"
fi

exec "$CODEX_BIN" resume --last -C "$REPO_DIR"
