#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
# shellcheck source=scripts/common/shlib.sh
. "$REPO_ROOT/scripts/common/shlib.sh"

ms_init "$REPO_ROOT"
ms_info "MicroSerial macOS bootstrap starting"
ms_info "Log file: $MS_LOG_FILE"

DO_INSTALL=-1
DO_BUILD=-1
DO_UNINSTALL=0

usage() {
    cat <<'USAGE'
MicroSerial macOS bootstrap

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

BREW_BIN=$(command -v brew || true)

RUST_MIN=1.74.0
RUSTUP_MIN=1.26.0
CMAKE_MIN=3.20.0
NINJA_MIN=1.10.0
PKGCFG_MIN=0.29.0
GIT_MIN=2.30.0
MAKE_MIN=4.2.0
CLANG_MIN=12.0.0
LLVM_MIN=13.0.0
CURL_MIN=7.70.0

ARCH=$(uname -m)
case $ARCH in
    arm64) EXPECTED_TARGET=aarch64-apple-darwin ;;
    x86_64) EXPECTED_TARGET=x86_64-apple-darwin ;;
    *) EXPECTED_TARGET=$ARCH-apple-darwin ;;
esac

check_prereqs() {
    if xcode-select -p >/dev/null 2>&1; then
        ms_append_report "[OK] Xcode Command Line Tools"
    else
        ms_append_report "[MISSING] Xcode Command Line Tools -> install: xcode-clt"
        ms_record_missing xcode-clt
    fi

    if [ -n "$BREW_BIN" ]; then
        ms_require_command "Homebrew" brew "brew --version" 'Homebrew \([0-9.]*\)' 3.0.0 homebrew
    else
        ms_append_report "[MISSING] Homebrew package manager"
        ms_record_missing homebrew
    fi

    ms_require_command "clang" clang "clang --version" 'Apple clang version \([0-9.]*\)' "$CLANG_MIN" xcode-clt
    ms_require_command "git" git "git --version" 'git version \([0-9.]*\)' "$GIT_MIN" git
    ms_require_command "CMake" cmake "cmake --version" 'cmake version \([0-9.]*\)' "$CMAKE_MIN" cmake
    ms_require_command "Ninja" ninja "ninja --version" '^\([0-9.]*\)$' "$NINJA_MIN" ninja
    ms_require_command "pkg-config" pkg-config "pkg-config --version" '^\([0-9.]*\)$' "$PKGCFG_MIN" pkg-config
    ms_require_command "GNU Make" gmake "gmake --version" 'GNU Make \([0-9.]*\)' "$MAKE_MIN" make
    ms_require_command "curl" curl "curl --version" '^curl \([0-9.]*\).*' "$CURL_MIN" curl

    if ! command -v gmake >/dev/null 2>&1 && command -v make >/dev/null 2>&1; then
        ms_append_report "[WARN] BSD make detected; installing gnu make recommended"
        ms_record_missing make
    fi

    if command -v llvm-config >/dev/null 2>&1; then
        ms_require_command "LLVM" llvm-config "llvm-config --version" '^\([0-9.]*\)$' "$LLVM_MIN" llvm
    else
        ms_append_report "[MISSING] llvm-config -> install: llvm"
        ms_record_missing llvm
    fi

    if pkg-config --exists libclang 2>/dev/null; then
        clang_ver=$(pkg-config --modversion libclang 2>/dev/null)
        if [ -n "$clang_ver" ]; then
            if ms_version_ge "$clang_ver" "$LLVM_MIN"; then
                ms_append_report "[OK] libclang $clang_ver"
            else
                ms_append_report "[OUTDATED] libclang $clang_ver (< $LLVM_MIN) -> install: llvm"
                ms_record_missing llvm
            fi
        else
            ms_append_report "[WARN] libclang version undetected"
        fi
    else
        ms_append_report "[MISSING] libclang development files -> install: llvm"
        ms_record_missing llvm
    fi

    if command -v g++ >/dev/null 2>&1; then
        gpp_banner=$(g++ --version 2>/dev/null | head -n 1)
        case $gpp_banner in
            "Apple clang"*)
                ms_append_report "[WARN] g++ resolves to Apple clang; installing Homebrew gcc recommended"
                ms_record_missing gcc
                ;;
            *)
                :
                ;;
        esac
    else
        ms_append_report "[MISSING] GNU C++ compiler -> install: gcc"
        ms_record_missing gcc
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
        if rustup target list --installed 2>/dev/null | grep -q "$EXPECTED_TARGET"; then
            ms_append_report "[OK] rustup target $EXPECTED_TARGET"
        else
            ms_append_report "[MISSING] rustup target $EXPECTED_TARGET -> install: rustup-target"
            ms_record_missing rustup-target
        fi
    fi
}

ensure_rustup() {
    ARCH_TARGET=$EXPECTED_TARGET
    if ! command -v rustup >/dev/null 2>&1; then
        if [ "$MS_OFFLINE" -eq 1 ]; then
            ms_die "rustup missing and offline mode requested"
        fi
        installer="$MS_CACHE_DIR/rustup-init-${ARCH_TARGET}"
        base_url="https://static.rust-lang.org/rustup/dist/${ARCH_TARGET}/rustup-init"
        ms_download_with_checksum "$base_url" "${base_url}.sha256" "$installer"
        ms_run_cmd "$installer" -y --profile minimal --no-modify-path
    fi

    ms_activate_rust_env
    ms_run_cmd rustup toolchain install stable
    ms_run_cmd rustup default stable
    ms_run_cmd rustup target add "$ARCH_TARGET"
}

add_pkg() {
    var=$1
    pkg=$2
    eval "current=\${$var:-}"
    case " $current " in
        *" $pkg "*) ;;
        *) current="$current $pkg" ;;
    esac
    current=$(printf "%s" "$current" | sed 's/^ *//')
    eval "$var=\"$current\""
}

macos_install_missing() {
    if [ "$MS_NEEDS_INSTALL" -eq 0 ] && [ "$MS_FORCE" -eq 0 ]; then
        ms_info "All prerequisites satisfied"
        return
    fi
    if [ "$MS_OFFLINE" -eq 1 ]; then
        missing=$(printf "%s" "$MS_MISSING" | tr '\n' ' ')
        ms_die "Offline mode requested but missing prerequisites: $missing"
    fi
    if [ -z "$BREW_BIN" ]; then
        ms_die "Homebrew not found. Install Homebrew manually (https://brew.sh) and re-run."
    fi

    BREW_PKGS=""
    NEED_RUST_INIT=0

    printf "%s" "$MS_MISSING" | sort | uniq | while IFS= read -r token; do
        [ -z "$token" ] && continue
        case $token in
            homebrew)
                ms_warn "Homebrew missing; manual installation required"
                ;;
            xcode-clt)
                ms_warn "Install Xcode Command Line Tools via 'xcode-select --install'"
                ;;
            git)
                add_pkg BREW_PKGS git
                ;;
            cmake)
                add_pkg BREW_PKGS cmake
                ;;
            ninja)
                add_pkg BREW_PKGS ninja
                ;;
            pkg-config)
                add_pkg BREW_PKGS pkg-config
                ;;
            make)
                add_pkg BREW_PKGS make
                ;;
            llvm)
                add_pkg BREW_PKGS llvm
                ;;
            gcc)
                add_pkg BREW_PKGS gcc
                ;;
            curl)
                add_pkg BREW_PKGS curl
                ;;
            rustup)
                NEED_RUST_INIT=1
                ;;
            rustup-toolchain)
                NEED_RUST_INIT=1
                ;;
            rustup-target)
                NEED_RUST_INIT=1
                ;;
            *)
                ms_warn "No package mapping for token $token"
                ;;
        esac
    done

    if [ -n "$BREW_PKGS" ]; then
        ms_info "Installing packages with Homebrew: $BREW_PKGS"
        ms_run_cmd brew update
        ms_run_cmd brew install $BREW_PKGS
    fi

    if command -v rustup >/dev/null 2>&1; then
        NEED_RUST_INIT=1
    fi

    if [ $NEED_RUST_INIT -eq 1 ] || [ $MS_FORCE -eq 1 ]; then
        ensure_rustup
    fi
}

perform_uninstall() {
    ms_info "Removing build directories"
    ms_run_shell "rm -rf '$REPO_ROOT/build'"
    ms_run_shell "rm -rf '$REPO_ROOT/gui/target'"
    ms_info "Retained toolchains; see docs for manual removal"
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
    if [ ! -f "$GUI_BIN" ]; then
        ms_die "Rust GUI binary not found at $GUI_BIN"
    fi
    ms_info "Rust GUI built: $GUI_BIN"
}

check_prereqs
ms_flush_report
ms_write_structured_log

if [ "$DO_UNINSTALL" -eq 1 ]; then
    perform_uninstall
    exit 0
fi

if [ "$DO_INSTALL" -eq 1 ]; then
    macos_install_missing
fi

if [ "$DO_BUILD" -eq 1 ]; then
    perform_build
fi

ms_info "macOS bootstrap complete"
