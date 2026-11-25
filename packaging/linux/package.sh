#!/usr/bin/env bash
#
# Vibe Terminal — Linux packaging script.
#
# Packages ONE build-matrix unit (category + arch) into a single set of
# artifacts (deb/rpm/tar.gz). The unit's binaries are read from --bin-dir,
# which is the per-arch staging dir output/<category>/<arch>/ (the inner
# scripts stage every component's binary there). For the desktop category the
# unit bundles whatever was staged: vterm+vtermc+vterm-server, plus
# vterm-screen-server and vterm-voice when those components were built.
#
# Artifact names follow the release scheme (consistent across formats; the
# flat tarball mirrors the .deb):
#   desktop   -> vibe-terminal-<ver>-linux-desktop-<arch>.*   (1 package)
#   headless  -> vibe-terminal-<ver>-linux-headless-<arch>.*  (1 package)
# where <arch> is the deb/tar.gz arch name (amd64/arm64/armhf/i386/riscv64);
# rpm uses its own arch names (x86_64/aarch64/armv7hl/i686/riscv64).
#
# Dist layout (two phases):
#   output/<category>/<arch>/       phase 1: raw binaries staged by arch
#   dist/linux/                     phase 2: the deb/rpm/tar.gz set, FLAT
#                                   (variant + arch are in the file names)
# where <category> is headless or desktop and <arch> is x64|x86|arm64|arm|riscv.
#
# glibc baseline: the unit declares the LOWEST floor among its binaries
# (--baseline, e.g. 2.27 for the desktop core). Components that need a newer
# glibc (screen ~2.31, voice ~2.35) are still bundled; their requirement is
# noted in the package Description so it can be communicated at distribution
# time.
#
# Package contents:
#   - deb/rpm install binaries to /usr/bin (FHS, package-manager owned).
#     A systemd *user* unit ships at /usr/lib/systemd/user/vterm-server.service
#     (the server runs as the invoking user, never as root). The desktop
#     variant also ships a vterm.desktop launcher entry and the hicolor app
#     icon so the GUI appears in the desktop's app menu.
#   - tar.gz is flat so install.sh can extract vterm-server to ~/.local/vterm.
#
# Usage:
#   packaging/linux/package.sh --skip-build --variant <desktop|headless>
#                              --arch <x64|x86|arm64|arm32|riscv64>
#                              --bin-dir <output/<category>/<arch>>
#                              --baseline <glibc floor, e.g. 2.27>
#                              [--version X.Y.Z]
#
# --arch names follow docker/build_inner.sh. --bin-dir points at the staged
# unit dir; the package bundles every vterm binary found there.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

VARIANT="headless"
SKIP_BUILD=0
VERSION=""
TARGET=""
BIN_DIR=""
BASELINE=""

while [ $# -gt 0 ]; do
    case "$1" in
        --skip-build) SKIP_BUILD=1 ;;
        --variant)    VARIANT="${2:?--variant needs headless}"; shift ;;
        --arch)       TARGET="${2:?--arch needs x64|x86|arm64|arm32|riscv64}"; shift ;;
        --bin-dir)    BIN_DIR="${2:?--bin-dir needs a directory}"; shift ;;
        --version)    VERSION="${2:?--version needs a value}"; shift ;;
        --baseline)   BASELINE="${2:?--baseline needs a glibc floor like 2.27}"; shift ;;
        *) echo "unknown option: $1" >&2; exit 1 ;;
    esac
    shift
done

case "$VARIANT" in
    headless) ;;
    *) echo "--variant must be headless" >&2; exit 1 ;;
esac

if [ -z "$TARGET" ]; then
    case "$(uname -m)" in
        x86_64)  TARGET="x64" ;;
        aarch64) TARGET="arm64" ;;
        *) echo "unsupported host arch $(uname -m); pass --arch explicitly" >&2; exit 1 ;;
    esac
fi

# ARCH: artifact/tarball name component; DEB_ARCH/RPM_ARCH: package metadata.
case "$TARGET" in
    x64)   TRIPLE="x86_64-unknown-linux-gnu";      ARCH="x86_64";  DEB_ARCH="amd64";  RPM_ARCH="x86_64";  ARCH_SHORT="x64"   ;;
    x86)   TRIPLE="i686-unknown-linux-gnu";        ARCH="i686";    DEB_ARCH="i386";   RPM_ARCH="i686";    ARCH_SHORT="x86"   ;;
    arm64) TRIPLE="aarch64-unknown-linux-gnu";     ARCH="aarch64"; DEB_ARCH="arm64";  RPM_ARCH="aarch64"; ARCH_SHORT="arm64" ;;
    arm32) TRIPLE="armv7-unknown-linux-gnueabihf"; ARCH="armv7";   DEB_ARCH="armhf";  RPM_ARCH="armv7hl"; ARCH_SHORT="arm"   ;;
    riscv64) TRIPLE="riscv64gc-unknown-linux-gnu"; ARCH="riscv64"; DEB_ARCH="riscv64"; RPM_ARCH="riscv64"; ARCH_SHORT="riscv" ;;
    *)     echo "--arch must be x64|x86|arm64|arm32|riscv64" >&2; exit 1 ;;
esac

# Default glibc floor when --baseline is not given (desktop core / headless).
[ -n "$BASELINE" ] || BASELINE="2.27"

if [ -z "$VERSION" ]; then
    VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
fi
[ -n "$VERSION" ] || { echo "could not determine version; pass --version" >&2; exit 1; }

log() { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31mError:\033[0m %s\n' "$*" >&2; exit 1; }

case "$VARIANT" in
    headless) PKG_NAME="vterm-headless" ;;
    desktop)  PKG_NAME="vterm" ;;
esac
# ART_BASE is the release artifact FILE name prefix; the installed package
# keeps its distro name (PKG_NAME) so upgrades from older releases keep
# working. deb/tar.gz append the deb arch, rpm the rpm arch:
#   vibe-terminal-<ver>-linux-<variant>-<debarch>.{deb,tar.gz}
#   vibe-terminal-<ver>-linux-<variant>-<rpmarch>.rpm
ART_BASE="vibe-terminal-${VERSION}-linux-${VARIANT}"
SERVER_PKG=1   # both ship vterm-server (and a systemd user unit)
CATEGORY="$VARIANT"

# ── Build (only when not --skip-build; the docker entry points always skip) ─
if [ "$SKIP_BUILD" -eq 0 ]; then
    log "Building webui"
    (cd webui && npm install && npm run build)
    if [ -z "$BIN_DIR" ]; then
        BIN_DIR="${CARGO_TARGET_DIR:-$ROOT/target}/$TRIPLE/release"
    fi
    log "Building vterm-server + vtermc (headless, release, $TRIPLE)"
    cargo build --release --target "$TRIPLE" \
        -p vterm-server --no-default-features -p vterm-cli
fi

[ -n "$BIN_DIR" ] || die "--bin-dir is required (the staged unit dir)"
[ -d "$BIN_DIR" ] || die "bin-dir not found: $BIN_DIR"

# The unit bundles every vterm binary staged into --bin-dir. A whitelist keeps
# stray files out of the package if a raw target dir is pointed at.
KNOWN_BINARIES="vterm-server vtermc"
BINARIES=""
for b in $KNOWN_BINARIES; do
    # Git Bash on Windows does not mark extensionless Linux binaries as
    # executable; install -m 0755 below sets the packaged file mode.
    [ -f "$BIN_DIR/$b" ] && BINARIES="$BINARIES $b"
done
BINARIES="$(echo "$BINARIES" | xargs)"
[ -n "$BINARIES" ] || die "no vterm binaries found in $BIN_DIR (build/stage first)"
log "binaries in unit: $BINARIES"

# ── Layout ────────────────────────────────────────────────────────────────
# Phase 1 (staging) is done by the inner scripts, which copy each component's
# binary into output/<category>/<arch>/ (= $BIN_DIR). We package directly from
# there into dist/linux/, FLAT: the artifact file names carry the variant and
# arch (vibe-terminal-<ver>-linux-<variant>-<arch>.*), so no subfolders.
STAGE_BIN="$BIN_DIR"
DIST="$ROOT/dist/linux"
DIST_REL="dist/linux"
mkdir -p "$DIST"
rm -f "$DIST/${ART_BASE}-${DEB_ARCH}.tar.gz" \
      "$DIST/${ART_BASE}-${DEB_ARCH}.deb" \
      "$DIST/${ART_BASE}-${RPM_ARCH}.rpm"
STAGE="$DIST/stage-$CATEGORY-$ARCH"
rm -rf "$STAGE"
mkdir -p "$DIST" "$STAGE/usr/bin" \
         "$STAGE/usr/share/doc/$PKG_NAME"
[ "$SERVER_PKG" -eq 1 ] && mkdir -p "$STAGE/usr/lib/systemd/user"

for bin in $BINARIES; do
    install -m 0755 "$STAGE_BIN/$bin" "$STAGE/usr/bin/$bin"
done
# Third-party notices, generated into webui/public by the webui build
# (npm run generate:licenses); also embedded in vterm-server under
# /third-party-notices.txt.
if [ -f "$ROOT/webui/public/third-party-notices.txt" ]; then
    install -m 0644 "$ROOT/webui/public/third-party-notices.txt" \
        "$STAGE/usr/share/doc/$PKG_NAME/third-party-notices.txt"
else
    die "webui/public/third-party-notices.txt missing - run the webui build (npm run build) first"
fi

# System-provided unit; users opt in with `systemctl --user enable --now
# vterm-server`.
if [ "$SERVER_PKG" -eq 1 ]; then
    cat > "$STAGE/usr/lib/systemd/user/vterm-server.service" <<EOF
[Unit]
Description=Vibe Terminal Server
After=network.target

[Service]
ExecStart=/usr/bin/vterm-server
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF
    chmod 0644 "$STAGE/usr/lib/systemd/user/vterm-server.service"
fi

# App launcher entry + icon so the installed GUI appears in the desktop's app
# menu (desktop variant only; headless ships no GUI binary).
if [ "$VARIANT" = "desktop" ]; then
    mkdir -p "$STAGE/usr/share/applications" \
             "$STAGE/usr/share/icons/hicolor/256x256/apps"
    cat > "$STAGE/usr/share/applications/vterm.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Vibe Terminal
GenericName=Terminal
Comment=AURGA Vibe Terminal
Exec=/usr/bin/vterm %U
Icon=vterm
StartupWMClass=vterm
Terminal=false
Categories=Development;System;TerminalEmulator;
StartupNotify=true
EOF
    chmod 0644 "$STAGE/usr/share/applications/vterm.desktop"
    install -m 0644 crates/vterm/assets/app.png \
        "$STAGE/usr/share/icons/hicolor/256x256/apps/vterm.png"
fi

# ── Removal hook (deb prerm / rpm %preun) ─────────────────────────────────
# Removal deletes the binaries, so stop everything that is still running.
# vterm-server.service uses Restart=on-failure: the systemd user unit must be
# stopped for every active user manager FIRST, or pkill would just trigger a
# restart mid-removal. Runs only on real removal (deb: prerm remove; rpm:
# %preun with $1=0) — never on upgrade, where processes keep running and pick
# up the new binaries.
STOP_BODY='for dir in /run/user/*; do
    uid="${dir#/run/user/}"
    [ -S "$dir/bus" ] || continue
    name="$(getent passwd "$uid" | cut -d: -f1)"
    [ -n "$name" ] || continue
    runuser -u "$name" -- env DBUS_SESSION_BUS_ADDRESS="unix:path=$dir/bus" \
        XDG_RUNTIME_DIR="$dir" systemctl --user stop vterm-server.service \
        >/dev/null 2>&1 || true
done
# Kill whatever is left: manually started processes, non-systemd sessions.
for b in vterm-server vterm vterm-screen-server vterm-voice; do
    pkill -x "$b" >/dev/null 2>&1 || true
done'

# ── Dependencies (hand-pinned, cross-arch safe) ───────────────────────────
# dpkg-shlibdeps would report the build host's glibc for cross-built binaries
# and needs each arch's .so present, so instead we pin floors by hand from the
# real DT_NEEDED of each component. The unit floor is $BASELINE (lowest core
# glibc); per-component extras are added only when that binary is present.
has() { case " $BINARIES " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
verlt() { awk -v a="$1" -v b="$2" 'BEGIN{print (a<b)?1:0}'; }

case "$VARIANT" in
    headless)
        # vterm-server (no-default-features) + vtermc
        DEPENDS="libc6 (>= $BASELINE), libudev1, libgcc-s1"
        ;;
    desktop)
        # vterm (GUI) + vterm-server (GUI features) + vtermc
        DEPENDS="libc6 (>= $BASELINE), libstdc++6, libfontconfig1, libx11-6, \
libxext6, libgl1, libxrandr2, libxrender1, libasound2, libudev1, libgcc-s1"
        if has vterm-screen-server; then
            DEPENDS="$DEPENDS, libpipewire-0.3-0, libdbus-1-3, libxdo3, libxkbcommon0"
        fi
        # vterm-voice needs no extra system libraries (CUDA at runtime).
        ;;
esac
# Collapse the manual line-wrap into a single space-separated string.
DEPENDS="$(echo "$DEPENDS" | tr '\n' ' ' | sed 's/  */ /g')"

# Extra Description lines: flag components that need a newer glibc than the
# unit floor (so it can be stated at distribution time), plus server autostart.
EXTRA_LINES=()
if [ "$VARIANT" = "desktop" ]; then
    if has vterm-screen-server && [ "$(verlt "$BASELINE" 2.31)" = "1" ]; then
        EXTRA_LINES+=(" vterm-screen-server requires glibc >= 2.31 (PipeWire 0.3).")
    fi
    if has vterm-voice && [ "$(verlt "$BASELINE" 2.35)" = "1" ]; then
        EXTRA_LINES+=(" vterm-voice requires glibc >= 2.35 and an NVIDIA GPU (driver >= 580).")
    fi
fi
EXTRA_LINES+=("Headless autostart (runs as your user, never root):")
EXTRA_LINES+=("  systemctl --user enable --now vterm-server")
# Debian control continuation lines MUST start with a space.
EXTRA_DESC="$(printf '%s\n' "${EXTRA_LINES[@]}" | sed 's/^/ /')"

DESCRIPTION="Vibe Terminal (${VARIANT}) - remote terminal server and tools"
RPM_SUMMARY="Vibe Terminal (${VARIANT}) - remote terminal server and tools"
# RPM description is plain text (no forced continuation indent).
RPM_DESC="Binaries: $(echo $BINARIES | tr ' ' ',').
$(printf '%s\n' "${EXTRA_LINES[@]}")"

# ── tar.gz (flat, install.sh-compatible, name mirrors the .deb) ──────────
TARBALL="${ART_BASE}-${DEB_ARCH}.tar.gz"
TAR_DIR="$DIST/tar-$CATEGORY-$ARCH"
rm -rf "$TAR_DIR"
mkdir -p "$TAR_DIR"
for bin in $BINARIES; do
    install -m 0755 "$STAGE_BIN/$bin" "$TAR_DIR/$bin"
done
tar -czf "$DIST/$TARBALL" -C "$TAR_DIR" $BINARIES
log "tar.gz: $DIST_REL/$TARBALL"

# ── .deb ─────────────────────────────────────────────────────────────────
if command -v dpkg-deb >/dev/null 2>&1; then
    DEB_ROOT="$DIST/deb-$CATEGORY-$ARCH"
    rm -rf "$DEB_ROOT"
    cp -a "$STAGE" "$DEB_ROOT"
    mkdir -p "$DEB_ROOT/DEBIAN"
    INSTALLED_SIZE="$(du -sk "$DEB_ROOT/usr" | cut -f1)"
    {
        cat <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Depends: ${DEPENDS}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Vibe Terminal <dev@vterminal.dev>
Description: ${DESCRIPTION}
 Binaries: $(echo $BINARIES | tr ' ' ',').
EOF
        # Debian control continuation lines MUST be indented with a space or
        # dpkg-deb reads the first word as a new field name.
        printf '%s\n' "$EXTRA_DESC" | sed 's/^/ /'
    } > "$DEB_ROOT/DEBIAN/control"
    {
        cat <<'EOF'
#!/bin/sh
# dpkg passes "remove" on real removal, "upgrade"/"deconfigure" otherwise;
# only stop processes when the package is actually going away.
case "${1:-}" in
    remove) ;;
    *) exit 0 ;;
esac
EOF
        printf '%s\n' "$STOP_BODY"
        echo 'exit 0'
    } > "$DEB_ROOT/DEBIAN/prerm"
    chmod 0755 "$DEB_ROOT/DEBIAN/prerm"
    DEB_FILE="${ART_BASE}-${DEB_ARCH}.deb"
    # -Zxz: build with xz (NOT the dpkg 1.23+ default zstd) so the .deb installs
    # on old systems whose dpkg predates zstd (dpkg < 1.20, Debian < 11) — they
    # fail with "unknown compression for member control.tar.zst". xz is
    # understood by every dpkg from Debian squeeze on.
    dpkg-deb --build --root-owner-group -Zxz "$DEB_ROOT" "$DIST/$DEB_FILE" >/dev/null
    log "deb:    $DIST_REL/$DEB_FILE"
else
    echo "dpkg-deb not found; skipping .deb" >&2
fi

# ── .rpm ─────────────────────────────────────────────────────────────────
if command -v rpmbuild >/dev/null 2>&1; then
    RPM_TOP="$DIST/rpmbuild-$CATEGORY-$ARCH"
    rm -rf "$RPM_TOP"
    mkdir -p "$RPM_TOP"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

    RPM_FILES=""
    for bin in $BINARIES; do
        RPM_FILES="$RPM_FILES
/usr/bin/$bin"
    done

    cat > "$RPM_TOP/SPECS/$PKG_NAME.spec" <<EOF
Name:           ${PKG_NAME}
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        ${RPM_SUMMARY}
License:        BUSL-1.1
# No BuildArch: the package arch comes from rpmbuild --target. A BuildArch
# line would be checked against the build host arch and fails for cross-built
# packages on rpm 4.14 ("No compatible architectures found for build").
# Cross-built packages: skip the post-install brp scripts (brp-strip cannot
# process foreign-arch binaries) and the debug package.
%define debug_package %{nil}
%global __os_install_post %{nil}

%description
${RPM_DESC}

%install
mkdir -p %{buildroot}
cp -a "$STAGE"/. %{buildroot}/

%files
${RPM_FILES}
EOF
    [ "$SERVER_PKG" -eq 1 ] && \
        printf '/usr/lib/systemd/user/vterm-server.service\n' >> "$RPM_TOP/SPECS/$PKG_NAME.spec"
    printf '/usr/share/doc/%s/third-party-notices.txt\n' "$PKG_NAME" >> "$RPM_TOP/SPECS/$PKG_NAME.spec"
    if [ "$VARIANT" = "desktop" ]; then
        printf '/usr/share/applications/vterm.desktop\n/usr/share/icons/hicolor/256x256/apps/vterm.png\n' \
            >> "$RPM_TOP/SPECS/$PKG_NAME.spec"
    fi
    # %preun: stop running vterm processes, but only on final removal
    # ($1=0); on upgrade ($1>=1) they keep running into the new version.
    cat >> "$RPM_TOP/SPECS/$PKG_NAME.spec" <<EOF

%preun
if [ "\$1" -eq 0 ]; then
$(printf '%s\n' "$STOP_BODY" | sed 's/^/    /')
fi
EOF

    # _binary_payload w6.xzdio: compress the rpm payload with xz (NOT the new
    # rpm 5.x default zstd) so the .rpm installs on old rpm (< 5.0, no zstd).
    # Level 6 balances size vs. packaging speed for the ~94 MB vterm-server.
    rpmbuild -bb --target "${RPM_ARCH}-linux" \
        --define "_topdir $RPM_TOP" \
        --define "_binary_payload w6.xzdio" \
        "$RPM_TOP/SPECS/$PKG_NAME.spec" >/dev/null
    RPM_OUT="$(find "$RPM_TOP/RPMS" -name '*.rpm' | head -n 1)"
    [ -n "$RPM_OUT" ] || die "rpmbuild produced no .rpm"
    RPM_FILE="${ART_BASE}-${RPM_ARCH}.rpm"
    mv "$RPM_OUT" "$DIST/$RPM_FILE"
    log "rpm:    $DIST_REL/$RPM_FILE"
else
    echo "rpmbuild not found; skipping .rpm" >&2
fi

# Clean up temp dirs (guard with ${:-}/if so set -e/-u don't trip when a tool
# was absent and the var never set).
rm -rf "$TAR_DIR" "$STAGE"
if [ -n "${DEB_ROOT:-}" ]; then
    rm -rf "$DEB_ROOT"
fi
if [ -n "${RPM_TOP:-}" ]; then
    rm -rf "$RPM_TOP"
fi
