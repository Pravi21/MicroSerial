# CI Examples

The workflow in `.github/workflows/bootstrap-ci.yml` demonstrates a cross-platform build using the bootstrap scripts. Key points:

* Matrix over `ubuntu-latest`, `macos-latest`, and `windows-latest`.
* Uses cache directories for Cargo and CMake to accelerate repeated builds.
* Runs `--audit-only` first to surface toolchain drift without modifying the environment, then performs a full bootstrap.
* Uploads build artifacts for inspection.

Integrate the workflow into your own pipelines or adapt the steps below:

```yaml
- name: Audit toolchains
  run: ./scripts/linux/bootstrap.sh --audit-only

- name: Bootstrap & build
  run: ./scripts/linux/bootstrap.sh --all --verbose
```

Windows runners should invoke PowerShell explicitly:

```yaml
- name: Windows build
  shell: pwsh
  run: |
    Set-ExecutionPolicy Bypass -Scope Process -Force
    ./scripts/windows/bootstrap.ps1 --all --verbose
```

When running in secure environments seed the package manager caches using the `MICROSERIAL_CACHE_DIR` environment variable and toggle `--offline` to validate offline operation.
