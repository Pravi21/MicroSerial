#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
# shellcheck source=scripts/common/shlib.sh
. "$REPO_ROOT/scripts/common/shlib.sh"

ms_init "$REPO_ROOT"
ms_info "MicroSerial Linux bootstrap starting"
ms_info "Log file: $MS_LOG_FILE"

DO_INSTALL=-1
DO_BUILD=-1
DO_UNINSTALL=0

usage() {
    cat <<'USAGE'
MicroSerial Linux bootstrap

Usage: bootstrap.sh [options]

  --audit-only       Run preflight checks only
  --install          Install missing prerequisites (no build)
  --all              Audit, install, and build
  --build            Build only (assumes prerequisites)
  --uninstall        Remove build artifacts and cached downloads
  --dry-run          Show actions without executing
  --force            Allow reinstall/upgrades even if versions match
  --offline          Do not attempt network access
  --verbose          Verbose logging
  --help             Show this help
USAGE
}

while [ $# -gt 0 ]; do
    case $1 in
        --audit-only)
            DO_INSTALL=0
            DO_BUILD=0
            ms_set_flag audit
            ;;
        --install|--bootstrap)
            DO_INSTALL=1
            if [ "$DO_BUILD" -eq -1 ]; then
                DO_BUILD=0
            fi
            ms_set_flag bootstrap
            ;;
        --build)
            DO_BUILD=1
            if [ "$DO_INSTALL" -eq -1 ]; then
                DO_INSTALL=0
            fi
            ms_set_flag build
            ;;
        --all)
            DO_INSTALL=1
            DO_BUILD=1
            ms_set_flag bootstrap
            ms_set_flag build
            ;;
        --uninstall)
            DO_UNINSTALL=1
            DO_INSTALL=0
            DO_BUILD=0
            ms_set_flag uninstall
            ;;
        --dry-run)
            ms_set_flag dry_run
            ;;
        --force)
            ms_set_flag force
            ;;
        --offline)
            ms_set_flag offline
            ;;
        --verbose)
            ms_set_flag verbose
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            ms_die "Unknown option: $1"
            ;;
    esac
    shift
done

if [ "$DO_INSTALL" -eq -1 ]; then
    DO_INSTALL=1
fi
if [ "$DO_BUILD" -eq -1 ]; then
    DO_BUILD=1
fi
if [ "$DO_UNINSTALL" -eq 1 ]; then
    DO_INSTALL=0
    DO_BUILD=0
fi

SUDO=
if command -v sudo >/dev/null 2>&1 && [ "$(id -u)" -ne 0 ]; then
    SUDO=sudo
fi

add_pkg() {
    var=$1
    pkg=$2
    eval "current=\${$var:-}"
    case " $current " in
        *" $pkg "*) ;;
        *) current="$current $pkg" ;;
    esac
    current=$(printf "%s" "$current" | sed 's/^ *//' )
    eval "$var=\"$current\""
}

run_pm() {
    if [ -n "$SUDO" ]; then
        ms_run_cmd "$SUDO" "$@"
    else
        ms_run_cmd "$@"
    fi
}

LINUX_PM=
if command -v apt-get >/dev/null 2>&1; then
    LINUX_PM=apt
elif command -v dnf >/dev/null 2>&1; then
    LINUX_PM=dnf
elif command -v pacman >/dev/null 2>&1; then
    LINUX_PM=pacman
else
    LINUX_PM=unknown
fi

RUST_MIN=1.74.0
RUSTUP_MIN=1.26.0
CMAKE_MIN=3.20.0
NINJA_MIN=1.10.0
PKGCFG_MIN=0.29.0
GIT_MIN=2.30.0
MAKE_MIN=4.2.0
GCC_MIN=9.0.0
GPP_MIN=9.0.0
LLVM_MIN=13.0.0
CLANGDEV_MIN=13.0.0
CURL_MIN=7.70.0

check_toolchains() {
    ms_require_command "git" git "git --version" 'git version \([0-9.]*\)' "$GIT_MIN" git
    ms_require_command "CMake" cmake "cmake --version" 'cmake version \([0-9.]*\)' "$CMAKE_MIN" cmake
    ms_require_command "Ninja" ninja "ninja --version" '^\([0-9.]*\)$' "$NINJA_MIN" ninja
    ms_require_command "pkg-config" pkg-config "pkg-config --version" '^\([0-9.]*\)$' "$PKGCFG_MIN" pkg-config
    ms_require_command "GNU Make" make "make --version" 'GNU Make \([0-9.]*\)' "$MAKE_MIN" build-essential
    ms_require_command "curl" curl "curl --version" '^curl \([0-9.]*\).*' "$CURL_MIN" curl
    if command -v gcc >/dev/null 2>&1; then
        ms_require_command "GCC" gcc "gcc -dumpfullversion" '^\([0-9.]*\)$' "$GCC_MIN" build-essential
    else
        ms_append_report "[MISSING] GCC -> install: build-essential"
        ms_record_missing build-essential
    fi
    if command -v clang >/dev/null 2>&1; then
        ms_require_command "Clang" clang "clang --version" 'clang version \([0-9.]*\)' "$GCC_MIN" clang
    else
        ms_append_report "[MISSING] Clang -> install: clang"
        ms_record_missing clang
    fi
    if command -v g++ >/dev/null 2>&1; then
        ms_require_command "G++" g++ "g++ -dumpfullversion" '^\([0-9.]*\)$' "$GPP_MIN" build-essential
    else
        ms_append_report "[MISSING] G++ compiler -> install: build-essential"
        ms_record_missing build-essential
    fi

    ms_require_command "rustup" rustup "rustup --version" 'rustup \([0-9.]*\)' "$RUSTUP_MIN" rustup
    if command -v rustc >/dev/null 2>&1; then
        ms_require_command "rustc" rustc "rustc --version" 'rustc \([0-9.]*\)' "$RUST_MIN" rustup
    else
        ms_append_report "[MISSING] rustc -> install: rustup"
        ms_record_missing rustup
    fi
    if command -v cargo >/dev/null 2>&1; then
        ms_require_command "cargo" cargo "cargo --version" 'cargo \([0-9.]*\)' "$RUST_MIN" rustup
    fi

    if command -v rustup >/dev/null 2>&1; then
        if rustup toolchain list --installed 2>/dev/null | grep -q '^stable'; then
            ms_append_report "[OK] rustup stable toolchain installed"
        else
            ms_append_report "[MISSING] rustup stable toolchain -> install: rustup-toolchain"
            ms_record_missing rustup-toolchain
        fi
        if rustup target list --installed 2>/dev/null | grep -q 'x86_64-unknown-linux-gnu'; then
            ms_append_report "[OK] rustup target x86_64-unknown-linux-gnu"
        else
            ms_append_report "[MISSING] rustup target x86_64-unknown-linux-gnu -> install: rustup-target-linux"
            ms_record_missing rustup-target-linux
        fi
    fi

    if pkg-config --exists libudev 2>/dev/null; then
        version=$(pkg-config --modversion libudev 2>/dev/null)
        if [ -n "$version" ]; then
            ms_append_report "[OK] libudev $version"
        else
            ms_append_report "[OK] libudev (version unknown)"
        fi
    else
        ms_append_report "[MISSING] libudev development files -> install: libudev"
        ms_record_missing libudev
    fi

    if command -v llvm-config >/dev/null 2>&1; then
        ms_require_command "LLVM" llvm-config "llvm-config --version" '^\([0-9.]*\)$' "$LLVM_MIN" llvm-dev
    else
        ms_append_report "[MISSING] llvm-config -> install: llvm-dev"
        ms_record_missing llvm-dev
    fi

    if pkg-config --exists libclang 2>/dev/null; then
        clang_ver=$(pkg-config --modversion libclang 2>/dev/null)
        if [ -n "$clang_ver" ]; then
            if ms_version_ge "$clang_ver" "$CLANGDEV_MIN"; then
                ms_append_report "[OK] libclang $clang_ver (>= $CLANGDEV_MIN)"
            else
                ms_append_report "[OUTDATED] libclang $clang_ver (< $CLANGDEV_MIN) -> install: libclang"
                ms_record_missing libclang
            fi
        else
            ms_append_report "[WARN] libclang version undetected"
        fi
    else
        ms_append_report "[MISSING] libclang development files -> install: libclang"
        ms_record_missing libclang
    fi
}

ensure_rustup() {
    ARCH=$(uname -m)
    case $ARCH in
        x86_64) RUSTUP_TRIPLE=x86_64-unknown-linux-gnu ;;
        aarch64) RUSTUP_TRIPLE=aarch64-unknown-linux-gnu ;;
        armv7l) RUSTUP_TRIPLE=armv7-unknown-linux-gnueabihf ;;
        armv6l) RUSTUP_TRIPLE=armv6-unknown-linux-gnueabihf ;;
        *) RUSTUP_TRIPLE="${ARCH}-unknown-linux-gnu" ;;
    esac

    if ! command -v rustup >/dev/null 2>&1; then
        if [ "$MS_OFFLINE" -eq 1 ]; then
            ms_die "rustup missing and offline mode requested"
        fi
        installer="$MS_CACHE_DIR/rustup-init-${RUSTUP_TRIPLE}"
        base_url="https://static.rust-lang.org/rustup/dist/${RUSTUP_TRIPLE}/rustup-init"
        ms_download_with_checksum "$base_url" "${base_url}.sha256" "$installer"
        ms_run_cmd "$installer" -y --profile minimal --no-modify-path
    fi

    ms_activate_rust_env
    ms_run_cmd rustup toolchain install stable
    ms_run_cmd rustup default stable
    ms_run_cmd rustup target add "$RUSTUP_TRIPLE"
}

linux_install_missing() {
    if [ "$MS_NEEDS_INSTALL" -eq 0 ] && [ "$MS_FORCE" -eq 0 ]; then
        ms_info "All prerequisites satisfied"
        return
    fi
    if [ "$MS_OFFLINE" -eq 1 ]; then
        missing=$(printf "%s" "$MS_MISSING" | tr '\n' ' ')
        ms_die "Offline mode requested but missing prerequisites: $missing"
    fi
    if [ "$LINUX_PM" = unknown ]; then
        ms_die "Unsupported package manager; install prerequisites manually"
    fi

    APT_PKGS=""
    DNF_PKGS=""
    PAC_PKGS=""
    NEED_RUSTUP=0

    printf "%s" "$MS_MISSING" | sort | uniq | while IFS= read -r token; do
        [ -z "$token" ] && continue
        case $token in
            git)
                add_pkg APT_PKGS git
                add_pkg DNF_PKGS git
                add_pkg PAC_PKGS git
                ;;
            cmake)
                add_pkg APT_PKGS cmake
                add_pkg DNF_PKGS cmake
                add_pkg PAC_PKGS cmake
                ;;
            ninja)
                add_pkg APT_PKGS ninja-build
                add_pkg DNF_PKGS ninja-build
                add_pkg PAC_PKGS ninja
                ;;
            pkg-config)
                add_pkg APT_PKGS pkg-config
                add_pkg DNF_PKGS pkgconfig
                add_pkg PAC_PKGS pkgconf
                ;;
            build-essential)
                add_pkg APT_PKGS build-essential
                add_pkg DNF_PKGS "@development-tools"
                add_pkg PAC_PKGS base-devel
                ;;
            clang)
                add_pkg APT_PKGS clang
                add_pkg DNF_PKGS clang
                add_pkg PAC_PKGS clang
                ;;
            llvm-dev)
                add_pkg APT_PKGS llvm-dev
                add_pkg DNF_PKGS llvm-devel
                add_pkg PAC_PKGS llvm
                ;;
            libclang)
                add_pkg APT_PKGS libclang-dev
                add_pkg DNF_PKGS clang-devel
                add_pkg PAC_PKGS clang
                ;;
            rustup)
                NEED_RUSTUP=1
                ;;
            rustup-toolchain)
                NEED_RUSTUP=1
                ;;
            rustup-target-linux)
                NEED_RUSTUP=1
                ;;
            libudev)
                add_pkg APT_PKGS libudev-dev
                add_pkg DNF_PKGS systemd-devel
                add_pkg PAC_PKGS systemd
                ;;
            curl)
                add_pkg APT_PKGS curl
                add_pkg DNF_PKGS curl
                add_pkg PAC_PKGS curl
                ;;
            *)
                ms_warn "No package mapping for token $token"
                ;;
        esac
    done

    case $LINUX_PM in
        apt)
            if [ -n "$APT_PKGS" ]; then
                ms_info "Installing packages: $APT_PKGS"
                run_pm apt-get update
                env_cmd="DEBIAN_FRONTEND=noninteractive"
                if [ -n "$SUDO" ]; then
                    ms_run_shell "$SUDO $env_cmd apt-get install -y $APT_PKGS"
                else
                    ms_run_shell "$env_cmd apt-get install -y $APT_PKGS"
                fi
            fi
            ;;
        dnf)
            if [ -n "$DNF_PKGS" ]; then
                ms_info "Installing packages: $DNF_PKGS"
                if [ -n "$SUDO" ]; then
                    ms_run_shell "$SUDO dnf install -y $DNF_PKGS"
                else
                    ms_run_shell "dnf install -y $DNF_PKGS"
                fi
            fi
            ;;
        pacman)
            if [ -n "$PAC_PKGS" ]; then
                ms_info "Installing packages: $PAC_PKGS"
                if [ -n "$SUDO" ]; then
                    ms_run_shell "$SUDO pacman -Sy --needed --noconfirm $PAC_PKGS"
                else
                    ms_run_shell "pacman -Sy --needed --noconfirm $PAC_PKGS"
                fi
            fi
            ;;
    esac

    if command -v rustup >/dev/null 2>&1; then
        NEED_RUSTUP=1
    fi

    if [ $NEED_RUSTUP -eq 1 ] || [ $MS_FORCE -eq 1 ]; then
        ensure_rustup
    fi
}

perform_uninstall() {
    ms_info "Removing build directories"
    ms_run_shell "rm -rf '$REPO_ROOT/build'"
    ms_run_shell "rm -rf '$REPO_ROOT/gui/target'"
    ms_info "Retained toolchains and system packages; see docs for manual removal"
}

perform_build() {
    BUILD_DIR="$REPO_ROOT/build/core"
    ms_info "Configuring C core via CMake"
    ms_run_cmd cmake -S "$REPO_ROOT/core" -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release
    ms_info "Building C core"
    ms_run_cmd cmake --build "$BUILD_DIR" --config Release
    CORE_ARTIFACT="$BUILD_DIR/libmicroserial_core.a"
    if [ ! -f "$CORE_ARTIFACT" ]; then
        ALT_ARTIFACT=$(find "$BUILD_DIR" -maxdepth 2 -name 'libmicroserial_core.a' | head -n 1 || true)
        if [ -n "$ALT_ARTIFACT" ]; then
            CORE_ARTIFACT="$ALT_ARTIFACT"
        fi
    fi
    if [ -f "$CORE_ARTIFACT" ]; then
        ms_info "C core built: $CORE_ARTIFACT"
    else
        ms_die "C core artifact not found"
    fi

    ms_info "Building Rust GUI"
    ms_run_cmd cargo build --manifest-path "$REPO_ROOT/gui/Cargo.toml" --release
    GUI_BIN="$REPO_ROOT/gui/target/release/microserial_gui"
    if [ ! -x "$GUI_BIN" ]; then
        ms_die "Rust GUI binary not found at $GUI_BIN"
    fi
    ms_info "Rust GUI built: $GUI_BIN"
}

check_toolchains
ms_flush_report
ms_write_structured_log

if [ "$DO_UNINSTALL" -eq 1 ]; then
    perform_uninstall
    exit 0
fi

if [ "$DO_INSTALL" -eq 1 ]; then
    linux_install_missing
fi

if [ "$DO_BUILD" -eq 1 ]; then
    perform_build
fi

ms_info "Linux bootstrap complete"
