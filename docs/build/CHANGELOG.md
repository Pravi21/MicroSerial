# Bootstrap Scripts Changelog

## 2026-05-11 — Hardened bootstrap + checksum pipeline

* Added checksum-verified download helpers for POSIX and PowerShell scripts.
* Expanded preflight coverage for LLVM/Clang dev headers, curl, and GNU toolchains across Linux, macOS, and Windows.
* Introduced `--install`/`--all` flags and documentation updates for the full audit→install→build workflow.

## 2025-09-19 — Initial cross-platform automation

* Added shared logging/version helpers (`scripts/common/`).
* Delivered Linux/macOS shell bootstraps and Windows PowerShell bootstrap.
* Implemented preflight audit matrix, conditional installs, and release builds.
* Published CI examples, threat model, troubleshooting, and quick-start documentation.
