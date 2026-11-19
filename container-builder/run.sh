#!/usr/bin/env bash
# Run deck-build.sh (or a command) in the Arch build container for PhotoCraft.
set -euo pipefail
cd "$(dirname "$0")/.."
IMAGE=vectorcraft-arch-builder
DECK_TARGET="${DECK_TARGET:-$HOME/.cache/photocraft-deck}"
mkdir -p "$DECK_TARGET" dist-deck

I18N_MOUNT=()
I18N_ENV=()
if [ -d "/home/deck/GIT/app/craft/craft-i18n" ]; then
  I18N_MOUNT=(-v "/home/deck/GIT/app/craft/craft-i18n:/craft-i18n:ro")
  I18N_ENV=(-e "CRAFT_I18N_DIR=/craft-i18n")
fi

ARGS=(--rm --userns=keep-id --security-opt label=disable
  -v "$PWD:/workspace:rw" -v "$DECK_TARGET:/target:rw" -v "$PWD/dist-deck:/out:rw"
  "${I18N_MOUNT[@]}"
  "${I18N_ENV[@]}"
  -v vectorcraft-cargo-registry:/opt/cargo/registry:rw
  -e CARGO_TARGET_DIR=/target
  -e DECK_CPU -e EXTRA_RUSTFLAGS)

if [ "${1:-}" = "bash" ]; then
  exec podman run -it "${ARGS[@]}" "$IMAGE" bash
elif [ "${1:-}" = "exec" ]; then
  shift
  exec podman run "${ARGS[@]}" "$IMAGE" "$@"
else
  exec podman run "${ARGS[@]}" "$IMAGE" bash /workspace/container-builder/deck-build.sh "$@"
fi
