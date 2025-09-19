# Bootstrap Threat Model

## Assets

* Developer workstations across Linux, macOS, and Windows.
* MicroSerial source tree, build artifacts, and signing keys (future).
* System toolchains and package manager trust stores.

## Adversaries

* **Network attackers** attempting to tamper with bootstrap downloads.
* **Malicious mirrors** serving compromised packages.
* **Local attackers** with access to package caches attempting path hijacking.

## Mitigations

* **Package manager trust.** All installations route through signed, TLS-protected package managers (winget, Homebrew, apt/dnf/pacman, Chocolatey, Scoop). Each manager validates checksums/signatures prior to install.
* **Preflight before download.** Scripts detect existing installations before invoking any network operation, preventing unnecessary downloads that could be intercepted.
* **Idempotent installation.** Repeat runs compare versions and only reinstall when forced, reducing exposure to supply-chain tampering.
* **Checksum and signature inheritance.** For direct installers (rustup-init, Visual Studio Build Tools) the delegated package manager verifies the publisher signature. We avoid raw `curl | sh` patterns.
* **Log transparency.** Every action and decision is logged to `build/logs/*.log`, enabling audit trails for incident response.
* **Offline enforcement.** The `--offline` flag aborts if a download would be required, supporting high-security environments with pre-populated caches.

## Residual Risks

* Compromise of upstream package feeds remains a systemic risk. Mitigate by pinning package versions in CI and mirroring trusted artifacts internally when possible.
* Visual Studio Build Tools installers are large and take time to verify; monitor install logs for failures and maintain offline installers for disaster recovery.
* macOS CLT installation requires GUI approval; ensure managed devices deploy it via MDM for unattended setups.
