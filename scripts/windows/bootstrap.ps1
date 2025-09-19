[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$ScriptArgs
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir '..\..')).Path
. (Join-Path $RepoRoot 'scripts/common/pslib.ps1')
Initialize-MicroSerial -Root $RepoRoot
Write-MSLog -Level 'INFO' -Message "MicroSerial Windows bootstrap starting"
Write-MSLog -Level 'INFO' -Message "Log file: $($MS.LogFile)"

$doInstall = $null
$doBuild = $null
$doUninstall = $false

if (-not $ScriptArgs) { $ScriptArgs = $args }

foreach ($arg in $ScriptArgs) {
    switch ($arg.ToLowerInvariant()) {
        '--audit-only' {
            $doInstall = $false
            $doBuild = $false
            Set-MSFlag 'audit'
        }
        '--bootstrap' {
            $doInstall = $true
            if (-not $doBuild.HasValue) { $doBuild = $false }
            Set-MSFlag 'bootstrap'
        }
        '--build' {
            $doBuild = $true
            if (-not $doInstall.HasValue) { $doInstall = $false }
            Set-MSFlag 'build'
        }
        '--uninstall' {
            $doUninstall = $true
            $doInstall = $false
            $doBuild = $false
            Set-MSFlag 'uninstall'
        }
        '--dry-run' { Set-MSFlag 'dry_run' }
        '--force' { Set-MSFlag 'force' }
        '--offline' { Set-MSFlag 'offline' }
        '--verbose' { Set-MSFlag 'verbose' }
        '--help' {
            Write-Host @'
MicroSerial Windows bootstrap

Usage: bootstrap.ps1 [options]
  --audit-only       Run preflight checks only
  --bootstrap        Install missing prerequisites (no build)
  --build            Build only (assumes prerequisites)
  --uninstall        Remove build artifacts and cached downloads
  --dry-run          Show actions without executing
  --force            Allow reinstall/upgrades even if versions match
  --offline          Do not attempt network access
  --verbose          Verbose logging
  --help             Show this help
'@
            exit 0
        }
        default { throw "Unknown option: $arg" }
    }
}

if (-not $doInstall.HasValue) { $doInstall = $true }
if (-not $doBuild.HasValue) { $doBuild = $true }
if ($doUninstall) { $doInstall = $false; $doBuild = $false }

$expectedTarget = 'x86_64-pc-windows-msvc'
if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') {
    $expectedTarget = 'aarch64-pc-windows-msvc'
}

function Get-VsInstallPath {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (-not (Test-Path $vswhere)) { return $null }
    $path = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($LASTEXITCODE -eq 0 -and $path) { return $path.Trim() }
    return $null
}

function Get-VsVersion {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (-not (Test-Path $vswhere)) { return $null }
    $version = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property catalog_productDisplayVersion 2>$null
    if ($LASTEXITCODE -eq 0 -and $version) { return $version.Trim() }
    return $null
}

function Get-WindowsSdkVersion {
    $key = 'HKLM:\SOFTWARE\Microsoft\Microsoft SDKs\Windows\v10.0'
    try {
        $props = Get-ItemProperty -Path $key -ErrorAction Stop
        return $props.ProductVersion
    } catch {
        return $null
    }
}

function Invoke-Preflight {
    Test-MSCommand -Name 'git' -Binary 'git' -VersionBlock { ([regex]::Match((git --version), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '2.30.0' -InstallRef 'git' | Out-Null
    Test-MSCommand -Name 'CMake' -Binary 'cmake' -VersionBlock { ([regex]::Match((cmake --version | Select-Object -First 1), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '3.20.0' -InstallRef 'cmake' | Out-Null
    Test-MSCommand -Name 'Ninja' -Binary 'ninja' -VersionBlock { ([regex]::Match((ninja --version), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '1.10.0' -InstallRef 'ninja' | Out-Null
    Test-MSCommand -Name 'pkg-config' -Binary 'pkg-config' -VersionBlock { ([regex]::Match((pkg-config --version), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '0.29.0' -InstallRef 'pkg-config' | Out-Null

    $makeCmd = Get-Command 'make' -ErrorAction SilentlyContinue
    if (-not $makeCmd) { $makeCmd = Get-Command 'mingw32-make' -ErrorAction SilentlyContinue }
    if ($makeCmd) {
        Test-MSCommand -Name 'make' -Binary $makeCmd.Source -VersionBlock { ([regex]::Match((& $makeCmd.Source --version | Select-Object -First 1), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '4.2.0' -InstallRef 'make' | Out-Null
    } else {
        Add-MSReport '[MISSING] GNU Make -> install: make'
        Add-MissingRequirement 'make'
    }

    $vsPath = Get-VsInstallPath
    $vsVersion = Get-VsVersion
    if ($vsPath -and $vsVersion) {
        $MS.VsInstallPath = $vsPath
        $devCmd = Join-Path $vsPath 'Common7/Tools/VsDevCmd.bat'
        if (Test-Path $devCmd) {
            $MS.VsDevCmd = $devCmd
            Add-MSReport "[OK] MSVC Build Tools $vsVersion"
        } else {
            Add-MSReport "[WARN] VsDevCmd not found in $vsPath"
            Add-MissingRequirement 'vs-buildtools'
        }
    } else {
        Add-MSReport '[MISSING] MSVC Build Tools -> install: vs-buildtools'
        Add-MissingRequirement 'vs-buildtools'
    }

    $sdkVersion = Get-WindowsSdkVersion
    if ($sdkVersion) {
        if (Test-VersionAtLeast -Current $sdkVersion -Minimum '10.0.19041.0') {
            Add-MSReport "[OK] Windows SDK $sdkVersion"
        } else {
            Add-MSReport "[OUTDATED] Windows SDK $sdkVersion (< 10.0.19041.0) -> install: vs-buildtools"
            Add-MissingRequirement 'vs-buildtools'
        }
    } else {
        Add-MSReport '[MISSING] Windows SDK -> install: vs-buildtools'
        Add-MissingRequirement 'vs-buildtools'
    }

    Test-MSCommand -Name 'rustup' -Binary 'rustup' -VersionBlock { ([regex]::Match((rustup --version), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '1.26.0' -InstallRef 'rustup' | Out-Null
    if (Get-Command 'rustc' -ErrorAction SilentlyContinue) {
        Test-MSCommand -Name 'rustc' -Binary 'rustc' -VersionBlock { ([regex]::Match((rustc --version), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '1.74.0' -InstallRef 'rustup' | Out-Null
    } else {
        Add-MSReport '[MISSING] rustc -> install: rustup'
        Add-MissingRequirement 'rustup'
    }
    if (Get-Command 'cargo' -ErrorAction SilentlyContinue) {
        Test-MSCommand -Name 'cargo' -Binary 'cargo' -VersionBlock { ([regex]::Match((cargo --version), '[0-9]+(\.[0-9]+)+')).Value } -Minimum '1.74.0' -InstallRef 'rustup' | Out-Null
    }

    if (Get-Command 'rustup' -ErrorAction SilentlyContinue) {
        $toolchains = rustup toolchain list --installed 2>$null
        if ($toolchains -match '^stable') {
            Add-MSReport '[OK] rustup stable toolchain installed'
        } else {
            Add-MSReport '[MISSING] rustup stable toolchain -> install: rustup-toolchain'
            Add-MissingRequirement 'rustup-toolchain'
        }
        $targets = rustup target list --installed 2>$null
        if ($targets -match [regex]::Escape($expectedTarget)) {
            Add-MSReport "[OK] rustup target $expectedTarget"
        } else {
            Add-MSReport "[MISSING] rustup target $expectedTarget -> install: rustup-target"
            Add-MissingRequirement 'rustup-target'
        }
    }
}

Invoke-Preflight
Show-MSReport
Write-MSReportToLog

if ($doUninstall) {
    Write-MSLog -Level 'INFO' -Message 'Removing build artifacts'
    Invoke-MSCommand -Command 'powershell' -Arguments @('-NoProfile','-Command',"Remove-Item -Recurse -Force '$RepoRoot\build' -ErrorAction SilentlyContinue")
    Invoke-MSCommand -Command 'powershell' -Arguments @('-NoProfile','-Command',"Remove-Item -Recurse -Force '$RepoRoot\gui\target' -ErrorAction SilentlyContinue")
    Write-MSLog -Level 'INFO' -Message 'Toolchains retained; see docs for full removal'
    exit 0
}

if (-not $doInstall) { $needsInstall = $false } else { $needsInstall = $true }

if ($needsInstall) {
    if ($MS.Offline -and $MS.Missing.Count -gt 0) {
        throw "Offline mode requested but prerequisites missing: $($MS.Missing.ToArray() -join ', ')"
    }

    $availableManagers = [ordered]@{
        winget = [bool](Get-Command 'winget' -ErrorAction SilentlyContinue)
        choco = [bool](Get-Command 'choco' -ErrorAction SilentlyContinue)
        scoop = [bool](Get-Command 'scoop' -ErrorAction SilentlyContinue)
    }

    function Install-Package {
        param(
            [string]$Token,
            [string]$WingetId,
            [string]$ChocoId,
            [string]$ScoopId
        )
        if ($WingetId -and $availableManagers.winget) {
            Invoke-MSCommand -Command 'winget' -Arguments @('install','--id',$WingetId,'--accept-package-agreements','--accept-source-agreements','--silent')
            return
        }
        if ($ChocoId -and $availableManagers.choco) {
            Invoke-MSCommand -Command 'choco' -Arguments @('install',$ChocoId,'-y','--no-progress')
            return
        }
        if ($ScoopId -and $availableManagers.scoop) {
            Invoke-MSCommand -Command 'scoop' -Arguments @('install',$ScoopId)
            return
        }
        throw "No installer available for $Token. Install manually."
    }

    foreach ($token in $MS.Missing) {
        switch ($token) {
            'git' { Install-Package -Token $token -WingetId 'Git.Git' -ChocoId 'git' -ScoopId 'git' }
            'cmake' { Install-Package -Token $token -WingetId 'Kitware.CMake' -ChocoId 'cmake' -ScoopId 'cmake' }
            'ninja' { Install-Package -Token $token -WingetId 'Ninja-build.Ninja' -ChocoId 'ninja' -ScoopId 'ninja' }
            'pkg-config' { Install-Package -Token $token -WingetId 'StrawberryPerl.StrawberryPerl' -ChocoId 'pkgconfiglite' -ScoopId 'pkg-config' }
            'make' { Install-Package -Token $token -WingetId 'GnuWin32.Make' -ChocoId 'make' -ScoopId 'make' }
            'rustup' { Install-Package -Token $token -WingetId 'Rustlang.Rustup' -ChocoId 'rustup.install' -ScoopId 'rustup' }
            'rustup-toolchain' { }
            'rustup-target' { }
            'vs-buildtools' {
                if ($availableManagers.winget) {
                    $override = '--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --add Microsoft.VisualStudio.Component.Windows10SDK.19041 --add Microsoft.VisualStudio.Component.Windows11SDK.22621'
                    Invoke-MSCommand -Command 'winget' -Arguments @('install','--id','Microsoft.VisualStudio.2022.BuildTools','--accept-package-agreements','--accept-source-agreements','--override',$override,'--silent')
                } elseif ($availableManagers.choco) {
                    Invoke-MSCommand -Command 'choco' -Arguments @('install','visualstudio2022buildtools','--package-parameters','"--includeRecommended --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.Windows10SDK.19041 --add Microsoft.VisualStudio.Component.Windows11SDK.22621"','-y','--no-progress')
                } else {
                    throw 'No package manager available to install Visual Studio Build Tools.'
                }
            }
            default { Write-MSLog -Level 'WARN' -Message "No installer mapping for token $token" }
        }
    }

    if (Get-Command 'rustup' -ErrorAction SilentlyContinue) {
        Invoke-MSCommand -Command 'rustup' -Arguments @('toolchain','install','stable')
        Invoke-MSCommand -Command 'rustup' -Arguments @('default','stable')
        Invoke-MSCommand -Command 'rustup' -Arguments @('target','add',$expectedTarget)
        $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
        if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $cargoBin })) {
            $env:PATH = "$cargoBin;$env:PATH"
        }
    }
}

function Invoke-WithVsEnv {
    param([string]$CommandLine)
    if (-not $MS.VsDevCmd) {
        throw 'VsDevCmd path not detected. Cannot build without MSVC environment.'
    }
    $wrapped = "call `"$($MS.VsDevCmd)`" -arch=x64 -host_arch=x64 && $CommandLine"
    Invoke-MSCommand -Command $wrapped -Shell
}

if ($doBuild) {
    $buildDir = Join-Path $RepoRoot 'build\core'
    $configure = "cmake -S `"$($RepoRoot)\core`" -B `"$buildDir`" -G Ninja -DCMAKE_BUILD_TYPE=Release"
    Invoke-WithVsEnv -CommandLine $configure
    $buildCore = "cmake --build `"$buildDir`" --config Release"
    Invoke-WithVsEnv -CommandLine $buildCore
    $coreArtifact = Join-Path $buildDir 'microserial_core.lib'
    if (-not (Test-Path $coreArtifact)) {
        $alt = Get-ChildItem -Path $buildDir -Recurse -Filter 'microserial_core.lib' -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($alt) { $coreArtifact = $alt.FullName }
    }
    if (-not (Test-Path $coreArtifact)) {
        throw "C core artifact not found in $buildDir"
    }
    Write-MSLog -Level 'INFO' -Message "C core built: $coreArtifact"

    $cargoCmd = "cargo build --manifest-path `"$($RepoRoot)\gui\Cargo.toml`" --release"
    Invoke-WithVsEnv -CommandLine $cargoCmd
    $guiBin = Join-Path $RepoRoot 'gui\target\release\microserial_gui.exe'
    if (-not (Test-Path $guiBin)) {
        throw "Rust GUI binary not found at $guiBin"
    }
    Write-MSLog -Level 'INFO' -Message "Rust GUI built: $guiBin"
}

Write-MSLog -Level 'INFO' -Message 'Windows bootstrap complete'
