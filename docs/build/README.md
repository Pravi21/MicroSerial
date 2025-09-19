# MicroSerial Bootstrap & Build Guide

The automation scripts in `scripts/<os>/bootstrap` deliver a one-command path from a clean machine to release-quality builds. They are
idempotent, audit first, and only install what is missing. Logs are written to `build/logs/` with timestamps for later review.

## Quick Start

| OS      | Command |
|---------|---------|
| Linux   | `./scripts/linux/bootstrap.sh --all` |
| macOS   | `./scripts/macos/bootstrap.sh --all` |
| Windows | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\windows\bootstrap.ps1 --all` |

All scripts default to **audit → install missing prerequisites → build**. Use the flags below to tailor behaviour:

* `--audit-only` – perform the full preflight without installing or building.
* `--install` – audit + install missing dependencies (skip build).
* `--all` – full pipeline: audit, install, and build in one run.
* `--build` – rebuild using existing toolchains.
* `--dry-run` – print actions without executing them.
* `--force` – reinstall/upgrade even when versions satisfy the minimums.
* `--offline` – abort installation if any download would be required.
* `--verbose` – increase log verbosity.
* `--uninstall` – remove `build/` and `gui/target/` artifacts (system toolchains are retained).

When successful the scripts print the release artifact paths:

* `build/core/libmicroserial_core.a` (or `microserial_core.lib` on Windows).
* `gui/target/release/microserial_gui[.exe]`.

Logs contain the structured preflight report along with every executed command. Direct downloads (for example the `rustup-init` bootstrapper used on developer machines without a packaged `rustup`) are fetched via HTTPS, validated against upstream SHA-256 sums, cached under `~/.cache/microserial` (or `%USERPROFILE%\.microserial\cache` on Windows), and re-verified on subsequent runs.

## Offline & cache-aware usage

Set `MICROSERIAL_CACHE_DIR=/path/to/cache` to point downloads to a pre-populated directory. Combine with `--offline` to validate
that all requirements are satisfied without reaching the network.

## Workflow Summary

1. **Audit** – detect compilers, SDKs, package managers, and Rust toolchains. No network access occurs.
2. **Conditional install** – missing tokens are mapped to the preferred package manager (winget/Homebrew/apt/dnf/pacman/scoop/choco). Any direct downloads go through the shared checksum helper for tamper detection before execution.
3. **Build orchestration** – C core via CMake+Ninja, Rust GUI via Cargo, executed after the toolchain environment is healthy.
4. **Verification** – artifact presence checks and cargo/cmake exit codes gate success.

See the rest of this folder for preflight matrices, threat modelling, troubleshooting, and CI examples.

## Suggested improvements

* Mirror critical packages (Visual Studio Build Tools bootstrapper, Homebrew bottles) into an internal artifact repository to tighten supply-chain control.
* Add notarisation/code-signing hooks once certificates become available so release builds can be distributed directly to operators.
* Capture binary provenance (for example, in-toto attestations) in CI to trace every artifact back to a specific commit and bootstrap log.
