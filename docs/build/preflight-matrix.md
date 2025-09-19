# Preflight Requirement Matrix

| Component | Minimum Version | Linux Source | macOS Source | Windows Source |
|-----------|-----------------|--------------|--------------|----------------|
| git | 2.30 | apt/dnf/pacman | Homebrew | winget/choco/scoop |
| CMake | 3.20 | apt/dnf/pacman | Homebrew | winget/choco/scoop |
| Ninja | 1.10 | apt/dnf/pacman | Homebrew | winget/choco/scoop |
| pkg-config | 0.29 | apt/dnf/pacman (`pkg-config`/`pkgconf`) | Homebrew | winget (`StrawberryPerl`) or choco (`pkgconfiglite`) |
| GNU Make | 4.2 | build-essential/base-devel | Homebrew (`make`) | winget (`GnuWin32.Make`) or choco (`make`) |
| GCC/G++ | 9.0 | `build-essential` | Homebrew (`gcc`) | MSVC Build Tools (cl) |
| clang | 13.0 | `clang` | Xcode CLT / Homebrew `llvm` | LLVM.LLVM |
| llvm-config | 13.0 | `llvm-dev` | Homebrew `llvm` | LLVM.LLVM |
| libclang dev headers | 13.0 | `libclang-dev` / `clang-devel` / `clang` | Homebrew `llvm` | LLVM.LLVM |
| curl | 7.70 | apt/dnf/pacman | Homebrew | winget (`cURL.cURL`) or choco (`curl`) |
| rustup | 1.26 | packaged `rustup` or official installer | Homebrew `rustup-init` or checksum installer | winget (`Rustlang.Rustup`) or checksum installer |
| Rust toolchain | stable ≥ 1.74 | `rustup toolchain install stable` | `rustup toolchain install stable` | `rustup toolchain install stable` |
| Rust target | platform triple | `rustup target add x86_64/aarch64` | `rustup target add aarch64/x86_64-apple-darwin` | `rustup target add x86_64/aarch64-pc-windows-msvc` |
| Windows SDK | 10.0.19041 | n/a | n/a | Included with VS Build Tools |
| Xcode CLT | 14.0 | n/a | `xcode-select --install` | n/a |
| libudev headers | distro default | `libudev-dev/systemd-devel/systemd` | n/a | n/a |

The scripts report the detected versions alongside the minimums and record installation intents in the log.
