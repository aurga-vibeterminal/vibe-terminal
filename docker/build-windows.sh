#!/usr/bin/env bash
# Build Linux release artifacts from Windows Git Bash with Docker Desktop.
# Usage: bash docker/build-windows.sh [headless] [x64|x86|arm64|arm|riscv|all]
set -euo pipefail

BASEDIR=$(cd "$(dirname "$0")" && pwd)
SRC=$(cd "$BASEDIR/.." && pwd)
DOCKER_SRC=$(cygpath -m "$SRC")
CATEGORY="${1:-headless}"
ARCHES="${2:-x64}"
HOST_PROXY="${PROXY:-}"   # optional HTTP(S) proxy for downloads; no default
if [ -z "${CONTAINER_PROXY+x}" ]; then
    CONTAINER_PROXY="${HOST_PROXY//localhost/host.docker.internal}"
    CONTAINER_PROXY="${CONTAINER_PROXY//127.0.0.1/host.docker.internal}"
fi

# Docker Desktop must not receive Git Bash's translated /src paths.
export MSYS_NO_PATHCONV=1
# The Docker CLI runs on Windows, but build containers must use the host gateway.
# Only export when a proxy was given: empty HTTP_PROXY/http_proxy values can
# confuse tools that treat an empty string as a misconfiguration.
if [ -n "$HOST_PROXY" ]; then
    export HTTP_PROXY="$HOST_PROXY"
    export HTTPS_PROXY="$HOST_PROXY"
    export http_proxy="$HOST_PROXY"
    export https_proxy="$HOST_PROXY"
fi

# Builds previously launched from WSL leave the shared cache/staging dirs
# owned by the WSL uid (1000) with 0755 metadata, which the container user
# (this shell's uid, e.g. 197609 in Git Bash) cannot write through the bind
# mount — cargo dies with "Permission denied (os error 13)" in target-docker.
# Open those dirs up as root before building; a no-op when already writable.
CHMOD_ARGS=()
VOL_ARGS=()
for p in target-docker .docker-cache output dist; do
    if [ -d "$SRC/$p" ]; then
        CHMOD_ARGS+=("/src/$p")
    fi
done
# docker/build also mounts the host cargo git cache into the container.
CARGO_GIT="$(cygpath -m "$HOME/.cargo/git" 2>/dev/null || true)"
if [ -n "$CARGO_GIT" ] && [ -d "$CARGO_GIT" ]; then
    VOL_ARGS=(-v "$CARGO_GIT:/cargo-git")
    CHMOD_ARGS+=("/cargo-git")
fi
if [ "${#CHMOD_ARGS[@]}" -gt 0 ]; then
    CHMOD_IMAGE=vibe-terminal-build-env
    docker image inspect "$CHMOD_IMAGE" >/dev/null 2>&1 || CHMOD_IMAGE=ubuntu:18.04
    echo "=== relaxing permissions on docker-shared dirs (uid mismatch guard) ==="
    docker run --rm -u 0:0 \
        -v "$DOCKER_SRC:/src" \
        "${VOL_ARGS[@]}" \
        "$CHMOD_IMAGE" \
        chmod -R a+rwX "${CHMOD_ARGS[@]}"
fi

PROXY="$CONTAINER_PROXY" bash "$SRC/docker/build" "$CATEGORY" "$ARCHES"

# package.sh runs on the host in docker/build. Windows normally lacks dpkg-deb
# and rpmbuild, so repeat packaging in a build image to produce the full set.
if [ "$ARCHES" = "all" ]; then
    ARCHES="x64 x86 arm64 arm riscv"
fi

for SHORT in $ARCHES; do
    PACKAGE_IMAGE=vibe-terminal-build-env
    BASELINE=2.27
    case "$SHORT" in
        x64) TARGET=x64 ;;
        x86) TARGET=x86 ;;
        arm64) TARGET=arm64 ;;
        arm) TARGET=arm32 ;;
        riscv)
            TARGET=riscv64
            PACKAGE_IMAGE=vibe-terminal-build-desktop
            BASELINE=2.35
            ;;
        *) echo "unknown arch: $SHORT (use x64|x86|arm64|arm|riscv|all)" >&2; exit 1 ;;
    esac

    echo "=== packaging $CATEGORY/$SHORT inside $PACKAGE_IMAGE ==="
    docker run --rm \
        -u "$(id -u):$(id -g)" \
        -e HOME=/tmp \
        -v "$DOCKER_SRC:/src" \
        -w /src \
        "$PACKAGE_IMAGE" \
        bash packaging/linux/package.sh --skip-build \
            --variant "$CATEGORY" --arch "$TARGET" \
            --bin-dir "/src/output/$CATEGORY/$SHORT" \
            --baseline "$BASELINE"
done

echo "=== Linux packages are in $SRC/dist/linux ==="
