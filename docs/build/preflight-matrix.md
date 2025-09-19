# Preflight Requirement Matrix

| Component | Minimum Version | Linux Source | macOS Source | Windows Source |
|-----------|-----------------|--------------|--------------|----------------|
| git | 2.30 | apt/dnf/pacman | Homebrew | winget/choco/scoop |
| CMake | 3.20 | apt/dnf/pacman | Homebrew | winget/choco/scoop |
| Ninja | 1.10 | apt/dnf/pacman | Homebrew | winget/choco/scoop |
| pkg-config | 0.29 | apt/dnf/pacman (`pkg-config`/`pkgconf`) | Homebrew | winget (`StrawberryPerl`) or choco (`pkgconfiglite`) |
| GNU Make | 4.2 | build-essential/base-devel | Homebrew (`make`) | winget (`GnuWin32.Make`) or choco (`make`) |
| GCC/Clang | GCC 9 / Clang 12 | build-essential/clang | Xcode CLT (Apple clang 12) | MSVC Build Tools 2022 |
| rustup | 1.26 | apt/dnf/pacman (`rustup`) | Homebrew (`rustup-init`) | winget (`Rustlang.Rustup`) |
| Rust toolchain | stable ≥ 1.74 | `rustup toolchain install stable` | `rustup-init` stable profile | `rustup toolchain install stable` |
| Rust target | `x86_64-unknown-linux-gnu` | `rustup target add` | `aarch64/x86_64-apple-darwin` | `x86_64/aarch64-pc-windows-msvc` |
| Windows SDK | 10.0.19041 | n/a | n/a | Included with VS Build Tools |
| Xcode CLT | 14.0 | n/a | `xcode-select --install` | n/a |
| libudev headers | distro default | `libudev-dev/systemd-devel/systemd` | n/a | n/a |

The scripts report the detected versions alongside the minimums and record installation intents in the log.
