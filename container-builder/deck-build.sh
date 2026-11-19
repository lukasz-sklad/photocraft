#!/usr/bin/env bash
# Optimized, machine-specific PhotoCraft build for Steam Deck (AMD Zen 2 / Aerith / Sephiroth).
# Runs inside the Arch build container (vectorcraft-arch-builder).
set -euo pipefail

CPU="${DECK_CPU:-znver2}"
export RUSTFLAGS="-C target-cpu=$CPU -C link-arg=-fuse-ld=lld ${EXTRA_RUSTFLAGS:-}"
export CFLAGS="-O3 -march=$CPU -mtune=$CPU"
export CXXFLAGS="$CFLAGS"
export CARGO_PROFILE_RELEASE_OPT_LEVEL=3
export CARGO_PROFILE_RELEASE_LTO=thin
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=unwind
export CARGO_PROFILE_RELEASE_STRIP=none
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

PKGS=(-p photocraft -p photocraft-cli --features heif)
OUT="${OUT:-/out}"
mkdir -p "$OUT"

cd /workspace

echo "==> Building optimized PhotoCraft release binaries for $CPU..."
cargo build --release --locked "${PKGS[@]}"

echo "==> Packaging tarball..."
DIST="$OUT" packaging/linux/package.sh --skip-build --formats tar

echo "==> deck-build complete! Output files in $OUT:"
ls -lh "$OUT"
