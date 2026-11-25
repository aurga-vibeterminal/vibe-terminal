#!/bin/bash
# Runs INSIDE the build container (mounted at /src, current host user).
# Usage: build_inner.sh <x64|x86|arm64|arm32|riscv64>
set -euo pipefail

TARGET="${1:?usage: build_inner.sh <x64|x86|arm64|arm32|riscv64>}"
SRC=/src

case "$TARGET" in
    x64)
        TRIPLE=x86_64-unknown-linux-gnu
        ARCH_SHORT=x64
        ;;
    x86)
        TRIPLE=i686-unknown-linux-gnu
        ARCH_SHORT=x86
        export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=gcc
        export RUSTFLAGS="-C link-arg=-m32"
        export CC_i686_unknown_linux_gnu=gcc
        export CFLAGS_i686_unknown_linux_gnu=-m32
        export CXX_i686_unknown_linux_gnu=g++
        export CXXFLAGS_i686_unknown_linux_gnu=-m32
        export PKG_CONFIG_PATH=/usr/lib/i386-linux-gnu/pkgconfig
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    arm64)
        TRIPLE=aarch64-unknown-linux-gnu
        ARCH_SHORT=arm64
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
        export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
        export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
        export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
        export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    arm32)
        TRIPLE=armv7-unknown-linux-gnueabihf
        ARCH_SHORT=arm
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
        export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
        export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
        export AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar
        export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    riscv64)
        TRIPLE=riscv64gc-unknown-linux-gnu
        ARCH_SHORT=riscv
        export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=riscv64-linux-gnu-gcc
        export CC_riscv64gc_unknown_linux_gnu=riscv64-linux-gnu-gcc
        export CXX_riscv64gc_unknown_linux_gnu=riscv64-linux-gnu-g++
        export AR_riscv64gc_unknown_linux_gnu=riscv64-linux-gnu-ar
        export PKG_CONFIG_PATH=/usr/lib/riscv64-linux-gnu/pkgconfig
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    *)
        echo "unknown target: $TARGET (use x64|x86|arm64|arm32|riscv64)" >&2
        exit 1
        ;;
esac

echo "=== building headless target $TARGET ($TRIPLE) ==="
uname -m

export CARGO_TARGET_DIR="$SRC/target-docker/$TRIPLE"
export CARGO_HOME=/opt/cargo-cache
export HOME=/tmp

# Only present when docker/build ran with MOUNT_GIT_CREDENTIALS=1.
if [ -f /tmp/.git-credentials ]; then
    printf '[credential]\n\thelper = store\n' > /tmp/.gitconfig
fi
git config --global --add safe.directory /src

cd "$SRC"
RELEASE_DIR="$CARGO_TARGET_DIR/$TRIPLE/release"

if ! cargo metadata --locked --format-version 1 >/dev/null 2>&1; then
    echo "=== Cargo.lock out of sync with the workspace; regenerating ==="
    cargo metadata --format-version 1 >/dev/null
fi

cargo build --release --locked --target "$TRIPLE" \
    -p vterm-server --no-default-features -p vterm-cli

STAGE_DIR="$SRC/output/headless/$ARCH_SHORT"
mkdir -p "$STAGE_DIR"
install -m 0755 "$RELEASE_DIR/vterm-server" "$STAGE_DIR/vterm-server"
install -m 0755 "$RELEASE_DIR/vtermc" "$STAGE_DIR/vtermc"
echo "=== staged headless/$ARCH_SHORT: vterm-server, vtermc ==="
