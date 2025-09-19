#!/usr/bin/env sh
# Shared utilities for MicroSerial bootstrap scripts (POSIX shell compliant)
# shellcheck shell=sh

if [ "${MICROSERIAL_SHLIB_SOURCED:-0}" -eq 1 ]; then
    return 0
fi
MICROSERIAL_SHLIB_SOURCED=1

set -u

ms_init() {
    MS_ROOT=$1
    : "${MS_LOG_DIR:=$MS_ROOT/build/logs}"
    : "${MS_CACHE_DIR:=${HOME}/.cache/microserial}"
    MS_TIMESTAMP=$(date +%Y%m%d-%H%M%S 2>/dev/null || date +%s)
    MS_LOG_FILE=${MS_LOG_DIR}/$(basename "$0").${MS_TIMESTAMP}.log
    mkdir -p "$MS_LOG_DIR" "$MS_CACHE_DIR"
    : "${MS_VERBOSE:=0}"
    : "${MS_DRY_RUN:=0}"
    : "${MS_FORCE:=0}"
    : "${MS_OFFLINE:=0}"
    : "${MS_MODE:=bootstrap}"
    : "${MS_REPORT:=}"
    : "${MS_REPORT_STATUS:=0}"
    : "${MS_MISSING:=}"
    : "${MS_NEEDS_INSTALL:=0}"

    exec 3>>"$MS_LOG_FILE"

    if [ -t 1 ]; then
        MS_CLR_RESET=$(printf '\033[0m')
        MS_CLR_INFO=$(printf '\033[32m')
        MS_CLR_WARN=$(printf '\033[33m')
        MS_CLR_ERROR=$(printf '\033[31m')
        MS_CLR_ACTION=$(printf '\033[36m')
    else
        MS_CLR_RESET=""
        MS_CLR_INFO=""
        MS_CLR_WARN=""
        MS_CLR_ERROR=""
        MS_CLR_ACTION=""
    fi
}

ms_log() {
    level=$1
    shift
    message=$*
    color=""
    case $level in
        INFO) color=$MS_CLR_INFO ;;
        WARN) color=$MS_CLR_WARN ;;
        ERROR) color=$MS_CLR_ERROR ;;
        ACTION) color=$MS_CLR_ACTION ;;
        *) color=$MS_CLR_RESET ;;
    esac
    printf "%s[%s] %s%s\n" "$color" "$level" "$message" "$MS_CLR_RESET"
    printf "[%s] %s\n" "$level" "$message" >&3
}

ms_info() { ms_log INFO "$*"; }
ms_warn() { ms_log WARN "$*"; }
ms_error() { ms_log ERROR "$*"; }
ms_action() { ms_log ACTION "$*"; }

ms_die() {
    ms_error "$*"
    exit 1
}

ms_set_flag() {
    case $1 in
        verbose) MS_VERBOSE=1 ;;
        dry_run) MS_DRY_RUN=1 ;;
        force) MS_FORCE=1 ;;
        offline) MS_OFFLINE=1 ;;
        audit) MS_MODE=audit ;;
        build) MS_MODE=build ;;
        bootstrap) MS_MODE=bootstrap ;;
        uninstall) MS_MODE=uninstall ;;
    esac
}

ms_run_cmd() {
    cmd=$1
    shift
    if [ "$MS_DRY_RUN" -eq 1 ]; then
        ms_action "[dry-run] $cmd $*"
        return 0
    fi
    if [ "$MS_VERBOSE" -eq 1 ]; then
        ms_action "$cmd $*"
    else
        ms_info "Running: $cmd $*"
    fi
    "$cmd" "$@"
    status=$?
    if [ $status -ne 0 ]; then
        ms_die "Command failed ($status): $cmd $*"
    fi
}

ms_run_shell() {
    # Run arbitrary shell command string
    if [ "$MS_DRY_RUN" -eq 1 ]; then
        ms_action "[dry-run] $*"
        return 0
    fi
    if [ "$MS_VERBOSE" -eq 1 ]; then
        ms_action "$*"
        sh -c "$*"
    else
        ms_info "Running: $*"
        sh -c "$*"
    fi
    status=$?
    if [ $status -ne 0 ]; then
        ms_die "Command failed ($status): $*"
    fi
}

ms_append_report() {
    if [ -z "$MS_REPORT" ]; then
        MS_REPORT=$1
    else
        MS_REPORT=$(printf "%s\n%s" "$MS_REPORT" "$1")
    fi
}

ms_flush_report() {
    printf "\n=== Preflight Report ===\n"
    printf "%s\n" "$MS_REPORT"
    printf "=======================\n"
}

ms_version_ge() {
    v1=$1
    v2=$2
    awk -v v1="$v1" -v v2="$v2" '
    function norm(a, res, i, n) {
        n = split(a, res, /[._-]/)
        for (i = 1; i <= n; i++) {
            if (res[i] ~ /^[0-9]+$/) {
                res[i] = int(res[i])
            }
        }
        return n
    }
    BEGIN {
        norm(v1, A)
        norm(v2, B)
        max = (length(A) > length(B)) ? length(A) : length(B)
        for (i = 1; i <= max; i++) {
            av = (i in A) ? A[i] : 0
            bv = (i in B) ? B[i] : 0
            if (av > bv) exit 0
            if (av < bv) exit 1
        }
        exit 0
    }
    '
}

ms_detect_version() {
    # Usage: ms_detect_version "command" "args" "regex"
    cmd=$1
    shift
    args=$1
    shift
    regex=$1
    if ! command -v "$cmd" >/dev/null 2>&1; then
        return 1
    fi
    output=$($cmd $args 2>/dev/null)
    printf "%s" "$output" | sed -n "s/$regex/\\1/p" | head -n 1
}

ms_record_missing() {
    item=$1
    case "\n$MS_MISSING\n" in
        *"\n$item\n"*) ;; # already recorded
        *) MS_MISSING="${MS_MISSING}${item}\n" ;;
    esac
    MS_NEEDS_INSTALL=1
}

ms_require_command() {
    name=$1
    binary=$2
    version_cmd=$3
    regex=$4
    min_version=$5
    install_ref=$6

    if ! command -v "$binary" >/dev/null 2>&1; then
        ms_append_report "[MISSING] $name ($binary) -> install: $install_ref"
        ms_record_missing "$install_ref"
        return 1
    fi

    version=$(sh -c "$version_cmd" 2>/dev/null | sed -n "s/$regex/\\1/p" | head -n 1)
    if [ -z "$version" ]; then
        ms_append_report "[WARN] $name version undetected -> install: $install_ref"
        return 0
    fi
    if ms_version_ge "$version" "$min_version"; then
        ms_append_report "[OK] $name $version (>= $min_version)"
        return 0
    fi
    ms_append_report "[OUTDATED] $name $version (< $min_version) -> install: $install_ref"
    ms_record_missing "$install_ref"
    return 1
}

ms_write_structured_log() {
    if [ -z "$MS_LOG_FILE" ]; then
        return
    fi
    {
        printf "==== Structured Report ====\n"
        printf "%s\n" "$MS_REPORT"
    } >>"$MS_LOG_FILE"
}
