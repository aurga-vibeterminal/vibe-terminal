# vibe-terminal Linux build environment (Ubuntu 18.04, glibc 2.27)
#
# One amd64 image that builds for four targets:
#   x64  -> x86_64-unknown-linux-gnu   (native)
#   x86  -> i686-unknown-linux-gnu     (gcc -m32)
#   arm64 -> aarch64-unknown-linux-gnu (gcc-aarch64-linux-gnu)
#   arm32 -> armv7-unknown-linux-gnueabihf (gcc-arm-linux-gnueabihf)
#
# Binaries link against glibc 2.27, so they run on Ubuntu 18.04 and newer.
# Build via ./build x64|x86|arm64|arm32|all from this directory.

FROM ubuntu:18.04

# HTTP(S) proxy reachable from the build (e.g. the host's local proxy when
# docker build runs with --network=host). Optional; used only for downloads
# (apt, rustup).
ARG PROXY=""

ENV DEBIAN_FRONTEND=noninteractive

# Ubuntu 18.04 (bionic) apt sources. Defaults are the canonical Ubuntu
# archives; pass a nearby mirror through docker/build (or --build-arg) to
# speed up downloads, e.g.
#   UBUNTU_MIRROR=http://mirrors.tuna.tsinghua.edu.cn/ubuntu
#   UBUNTU_PORTS_MIRROR=http://mirrors.tuna.tsinghua.edu.cn/ubuntu-ports
# arm64/armhf packages come from ubuntu-ports, i386 from the regular archive
# (bionic still ships i386). -updates/-security pockets are REQUIRED for
# multiarch: libc6 is Multi-Arch:same, so libc6:<arch> only co-installs with
# the exact version installed for amd64 — the base bionic pocket alone is
# too old.
ARG UBUNTU_MIRROR="http://archive.ubuntu.com/ubuntu"
ARG UBUNTU_PORTS_MIRROR="http://ports.ubuntu.com/ubuntu-ports"
RUN dpkg --add-architecture arm64 && \
    dpkg --add-architecture armhf && \
    dpkg --add-architecture i386 && \
    echo "deb [arch=amd64,i386] ${UBUNTU_MIRROR}/ bionic main restricted universe multiverse" > /etc/apt/sources.list && \
    echo "deb [arch=amd64,i386] ${UBUNTU_MIRROR}/ bionic-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=amd64,i386] ${UBUNTU_MIRROR}/ bionic-security main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] ${UBUNTU_PORTS_MIRROR}/ bionic main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] ${UBUNTU_PORTS_MIRROR}/ bionic-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] ${UBUNTU_PORTS_MIRROR}/ bionic-security main restricted universe multiverse" >> /etc/apt/sources.list

# Base build tools. (apt installs are split into separate layers: one giant
# install mixing multiarch foreign packages with the cross toolchains makes
# the bionic resolver give up with "held broken packages".)
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        libclang-dev \
        pkg-config \
        git \
        curl \
        ca-certificates \
        python3 \
        file \
    && rm -rf /var/lib/apt/lists/*

# Upgrade git to >= 2.31 from the git-core PPA (still serving bionic; the PPA
# entry is removed afterwards so later apt layers see the original sources).
# WHY: skia's tools/git-sync-deps (run by skia-bindings' build script) probes
# each dependency with `git rev-parse --path-format=relative --show-toplevel`,
# which bionic's stock git 2.17 rejects. The probe failure makes the script
# print "IS NOT TOP-LEVEL GIT DIRECTORY" and SILENTLY SKIP every dependency
# (exit 0), so the Skia build later dies in ninja on empty
# third_party/externals trees (e.g. icu/scripts/make_data_assembly.py missing).
RUN echo "deb [arch=amd64 trusted=yes] https://ppa.launchpadcontent.net/git-core/ppa/ubuntu bionic main" \
        > /etc/apt/sources.list.d/git-core-ppa.list && \
    http_proxy="$PROXY" https_proxy="$PROXY" \
        apt-get update -o Acquire::AllowInsecureRepositories=true && \
    http_proxy="$PROXY" https_proxy="$PROXY" \
        apt-get install -y --no-install-recommends git && \
    rm -f /etc/apt/sources.list.d/git-core-ppa.list && \
    rm -rf /var/lib/apt/lists/*

# Foreign-arch libc first: libc6 is Multi-Arch:same and only co-installs with
# the exact amd64 version (hence the -updates pockets above).
RUN apt-get update && apt-get install -y --no-install-recommends \
        libc6:arm64 \
        libc6:armhf \
        libc6:i386 \
    && rm -rf /var/lib/apt/lists/*

# Cross toolchains: arm64 / arm32 (armhf) / x86 (-m32).
# NOTE 1: two separate apt-get invocations — resolving all six packages in one
# command trips bionic's resolver ("held broken packages") when foreign-arch
# libc6 packages are installed; each group resolves cleanly on its own.
# NOTE 2: gcc-7-multilib instead of the gcc-multilib metapackage — the
# metapackages hard-Conflict every gcc-7-<arch>-linux-gnu cross compiler and
# silently uninstall them. gcc-7-multilib carries no Conflicts and provides
# the same -m32 bits (lib32gcc-7-dev, libc6-dev-i386, lib32stdc++-7-dev).
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        gcc-arm-linux-gnueabihf \
        g++-arm-linux-gnueabihf && \
    apt-get install -y --no-install-recommends \
        gcc-7-multilib \
        g++-7-multilib && \
    rm -rf /var/lib/apt/lists/*

# libudev for every target (serialport links against it)
RUN apt-get update && apt-get install -y --no-install-recommends \
        libudev-dev \
        libudev-dev:i386 \
        libudev-dev:arm64 \
        libudev-dev:armhf \
    && rm -rf /var/lib/apt/lists/*

# Packaging tools: rpmbuild for .rpm artifacts (dpkg-deb ships with the base
# system). tar/gzip are already present.
RUN apt-get update && apt-get install -y --no-install-recommends \
        rpm \
        unzip \
        xz-utils \
    && rm -rf /var/lib/apt/lists/*

# Desktop-variant system libraries for every target: the final
# vterm/vterm-server link needs X11/GL/fontconfig (skia x11 backend) and ALSA
# (vterm client's cpal). freetype itself is embedded by skia (embed-freetype
# feature), so no libfreetype-dev here. One layer per library family: the
# bionic multiarch resolver fails when they are mixed.
RUN apt-get update && apt-get install -y --no-install-recommends \
        libx11-dev \
        libx11-dev:i386 \
        libx11-dev:arm64 \
        libx11-dev:armhf \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends \
        libxrender-dev \
        libxrender-dev:i386 \
        libxrender-dev:arm64 \
        libxrender-dev:armhf \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends \
        libxrandr-dev \
        libxrandr-dev:i386 \
        libxrandr-dev:arm64 \
        libxrandr-dev:armhf \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends \
        libgl1-mesa-dev \
        libgl1-mesa-dev:i386 \
        libgl1-mesa-dev:arm64 \
        libgl1-mesa-dev:armhf \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends \
        libfontconfig-dev \
        libfontconfig-dev:i386 \
        libfontconfig-dev:arm64 \
        libfontconfig-dev:armhf \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends \
        libasound2-dev \
        libasound2-dev:i386 \
        libasound2-dev:arm64 \
        libasound2-dev:armhf \
    && rm -rf /var/lib/apt/lists/*

# Sysroot copies for bindgen's --sysroot during the cross desktop builds: the
# desktop container run bind-mounts an empty dir over /usr/<triple>/include
# (see docker/build) to neutralize rust-skia's harmful hard-coded -I flags, so
# libclang must get the C headers from these copies instead. The i686 one
# merges the multiarch shared dir with the i386 bits.
RUN mkdir -p /opt/sysroots/aarch64-linux-gnu/usr \
             /opt/sysroots/arm-linux-gnueabihf/usr \
             /opt/sysroots/i686-linux-gnu/usr/include && \
    cp -a /usr/aarch64-linux-gnu/include /opt/sysroots/aarch64-linux-gnu/usr/include && \
    cp -a /usr/arm-linux-gnueabihf/include /opt/sysroots/arm-linux-gnueabihf/usr/include && \
    cp -a /usr/include/. /opt/sysroots/i686-linux-gnu/usr/include/ && \
    rm -f /opt/sysroots/i686-linux-gnu/usr/include/bits \
          /opt/sysroots/i686-linux-gnu/usr/include/gnu && \
    cp -a /usr/include/i386-linux-gnu/. /opt/sysroots/i686-linux-gnu/usr/include/ && \
    mkdir -p /opt/sysroots/shared-include && \
    cp -a /usr/include/fontconfig /opt/sysroots/shared-include/fontconfig && \
    cp -a /usr/include/GL /opt/sysroots/shared-include/GL && \
    cp -a /usr/include/KHR /opt/sysroots/shared-include/KHR && \
    cp -a /usr/include/X11 /opt/sysroots/shared-include/X11 && \
    cp -a /usr/include/libudev.h /opt/sysroots/shared-include/libudev.h

# Modern toolchain for compiling Skia m145 from source (FORCE_SKIA_BUILD):
# bionic's clang 6 cannot do C++20, and skia-bindings needs a matching
# libclang for bindgen. LLVM 18.1.8 is the last release with official
# ubuntu-18.04 binaries. Downloads go through the build proxy.
RUN http_proxy="$PROXY" https_proxy="$PROXY" \
        curl -fsSL -o /tmp/llvm.tar.xz \
            https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/clang+llvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04.tar.xz && \
    mkdir -p /opt/llvm-18 && \
    tar -xJf /tmp/llvm.tar.xz -C /opt/llvm-18 --strip-components=1 && \
    rm /tmp/llvm.tar.xz && \
    cp /opt/llvm-18/include/x86_64-unknown-linux-gnu/c++/v1/__config_site \
       /opt/llvm-18/include/c++/v1/__config_site

# gn + ninja for the skia build (bionic ships neither a usable gn nor a
# new-enough ninja).
RUN http_proxy="$PROXY" https_proxy="$PROXY" \
        curl -fsSL -o /tmp/gn.zip \
            "https://chrome-infra-packages.appspot.com/dl/gn/gn/linux-amd64/+/latest" && \
    unzip -o /tmp/gn.zip -d /usr/local/bin && \
    rm /tmp/gn.zip && \
    http_proxy="$PROXY" https_proxy="$PROXY" \
        curl -fsSL -o /tmp/ninja.zip \
            https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-linux.zip && \
    unzip -o /tmp/ninja.zip -d /usr/local/bin && \
    rm /tmp/ninja.zip && \
    chmod +x /usr/local/bin/gn /usr/local/bin/ninja

# The skia build picks clang/clang++ (CC/CXX) and libclang (bindgen) from
# these; PATH puts LLVM 18 ahead of bionic's clang 6.
ENV PATH=/opt/llvm-18/bin:$PATH \
    LIBCLANG_PATH=/opt/llvm-18/lib

# zig (static binary) — the cross compiler for the arm64/x86/arm32 desktop
# builds. Its bundled clang 18 compiles Skia's C++20 against its own libc++
# (built per target on demand, statically linked), and its glibc-versioned
# targets (-target aarch64-linux-gnu.2.27) keep the binaries on the glibc
# 2.27 baseline. The LLVM tarball above only ships an x86_64 libc++, which
# is why the other architectures need zig.
RUN http_proxy="$PROXY" https_proxy="$PROXY" \
        curl -fsSL -o /tmp/zig.tar.xz \
            https://ziglang.org/download/0.13.0/zig-linux-x86_64-0.13.0.tar.xz && \
    tar -xJf /tmp/zig.tar.xz -C /opt && \
    mv /opt/zig-linux-x86_64-0.13.0 /opt/zig && \
    rm /tmp/zig.tar.xz
ENV PATH=/opt/zig:$PATH

# Rust toolchain: must match /rust-toolchain.toml (channel + rustfmt), plus
# the cross-compile std libraries. Re-run automatically otherwise, but a
# non-root runtime user could not write here, so preinstall everything.
# Downloads come from the official static server; when direct access is slow
# (or routed through a proxy), pass a mirror, e.g.
#   RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
#   RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
ARG RUSTUP_DIST_SERVER="https://static.rust-lang.org"
ARG RUSTUP_UPDATE_ROOT="https://static.rust-lang.org/rustup"
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH \
    RUSTUP_DIST_SERVER=${RUSTUP_DIST_SERVER} \
    RUSTUP_UPDATE_ROOT=${RUSTUP_UPDATE_ROOT}

RUN http_proxy="$PROXY" https_proxy="$PROXY" \
        curl -sSf https://sh.rustup.rs | sh -s -- -y \
            --default-toolchain 1.96.0 \
            --profile minimal \
            --component rustfmt \
    && rustup target add --toolchain 1.96.0 \
        i686-unknown-linux-gnu \
        aarch64-unknown-linux-gnu \
        armv7-unknown-linux-gnueabihf

# The container runs as the host user (docker run -u uid:gid); give it write
# access so rustup can install missing components/toolchains if the pinned
# rust-toolchain.toml ever moves ahead of this image.
RUN chmod -R a+rwX /opt/rustup /opt/cargo
