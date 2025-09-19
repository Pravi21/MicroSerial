if ($script:MicroSerialPsLibLoaded) { return }
$script:MicroSerialPsLibLoaded = $true

function Initialize-MicroSerial {
    param(
        [string]$Root
    )
    $global:MS = [ordered]@{
        Root = $Root
        LogDir = if ($env:MS_LOG_DIR) { $env:MS_LOG_DIR } else { Join-Path $Root 'build/logs' }
        CacheDir = if ($env:MS_CACHE_DIR) { $env:MS_CACHE_DIR } else { Join-Path $env:USERPROFILE '.microserial\cache' }
        Timestamp = (Get-Date -Format 'yyyyMMdd-HHmmss')
        DryRun = $false
        Verbose = $false
        Force = $false
        Offline = $false
        Mode = 'bootstrap'
        Report = New-Object System.Collections.Generic.List[string]
        Missing = New-Object System.Collections.Generic.HashSet[string]
    }
    if (-not (Test-Path $MS.LogDir)) { New-Item -ItemType Directory -Path $MS.LogDir | Out-Null }
    if (-not (Test-Path $MS.CacheDir)) { New-Item -ItemType Directory -Path $MS.CacheDir | Out-Null }
    $global:MS.LogFile = Join-Path $MS.LogDir ("{0}.{1}.log" -f (Split-Path $MyInvocation.PSCommandPath -Leaf), $MS.Timestamp)
    $null = New-Item -ItemType File -Path $MS.LogFile -Force
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
}

function Write-MSLog {
    param(
        [ValidateSet('INFO','WARN','ERROR','ACTION')]$Level,
        [string]$Message
    )
    $color = 'White'
    switch ($Level) {
        'INFO' { $color = 'Green' }
        'WARN' { $color = 'Yellow' }
        'ERROR' { $color = 'Red' }
        'ACTION' { $color = 'Cyan' }
    }
    Write-Host "[$Level] $Message" -ForegroundColor $color
    Add-Content -LiteralPath $MS.LogFile -Value "[$Level] $Message"
}

function Invoke-MSCommand {
    param(
        [string]$Command,
        [string[]]$Arguments,
        [switch]$Shell
    )
    if (-not $Arguments) { $Arguments = @() }
    $display = if ($Shell) { $Command } else { "{0} {1}" -f $Command, ($Arguments -join ' ') }
    if ($MS.DryRun) {
        Write-MSLog -Level 'ACTION' -Message "[dry-run] $display"
        return
    }
    Write-MSLog -Level 'ACTION' -Message $display
    if ($Shell) {
        & cmd.exe /c $Command
    } else {
        & $Command @Arguments
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed ($LASTEXITCODE): $display"
    }
}

function Test-VersionAtLeast {
    param(
        [string]$Current,
        [string]$Minimum
    )
    try {
        return ([version]($Current -replace '[^0-9\.]', '0')) -ge [version]$Minimum
    } catch {
        return $false
    }
}

function Add-MSReport {
    param([string]$Line)
    $MS.Report.Add($Line)
    Add-Content -LiteralPath $MS.LogFile -Value $Line
}

function Add-MissingRequirement {
    param([string]$Item)
    [void]$MS.Missing.Add($Item)
}

function Test-MSCommand {
    param(
        [string]$Name,
        [string]$Binary,
        [ScriptBlock]$VersionBlock,
        [string]$Minimum,
        [string]$InstallRef
    )
    $cmd = Get-Command $Binary -ErrorAction SilentlyContinue
    if (-not $cmd) {
        Add-MSReport "[MISSING] $Name ($Binary) -> install: $InstallRef"
        Add-MissingRequirement $InstallRef
        return $false
    }
    $version = & $VersionBlock 2>$null
    if (-not $version) {
        Add-MSReport "[WARN] $Name version undetected -> install: $InstallRef"
        return $true
    }
    if (Test-VersionAtLeast -Current $version -Minimum $Minimum) {
        Add-MSReport "[OK] $Name $version (>= $Minimum)"
        return $true
    }
    Add-MSReport "[OUTDATED] $Name $version (< $Minimum) -> install: $InstallRef"
    Add-MissingRequirement $InstallRef
    return $false
}

function Show-MSReport {
    Write-Host "`n=== Preflight Report ===" -ForegroundColor Cyan
    foreach ($line in $MS.Report) { Write-Host $line }
    Write-Host "=======================`n" -ForegroundColor Cyan
}

function Set-MSFlag {
    param([string]$Flag)
    switch ($Flag) {
        'verbose' { $MS.Verbose = $true }
        'dry_run' { $MS.DryRun = $true }
        'force' { $MS.Force = $true }
        'offline' { $MS.Offline = $true }
        'audit' { $MS.Mode = 'audit' }
        'build' { $MS.Mode = 'build' }
        'bootstrap' { $MS.Mode = 'bootstrap' }
        'uninstall' { $MS.Mode = 'uninstall' }
    }
}

function Write-MSReportToLog {
    Add-Content -LiteralPath $MS.LogFile -Value "==== Structured Report ===="
    foreach ($line in $MS.Report) { Add-Content -LiteralPath $MS.LogFile -Value $line }
}

function Invoke-MSDownload {
    param(
        [string]$Uri,
        [string]$ChecksumUri,
        [string]$Destination
    )

    $shaPath = "$Destination.sha256"
    if ($MS.Offline) {
        if (-not (Test-Path $Destination -PathType Leaf) -or -not (Test-Path $shaPath -PathType Leaf)) {
            throw "Offline mode active but cache missing for $(Split-Path -Leaf $Destination)"
        }
        $expected = ((Get-Content -LiteralPath $shaPath | Select-Object -First 1).Split(' ')[0]).Trim()
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Destination).Hash.ToLowerInvariant()
        if ($actual -ne $expected.ToLowerInvariant()) {
            throw "Cached checksum mismatch for $(Split-Path -Leaf $Destination)"
        }
        Write-MSLog -Level 'INFO' -Message "Checksum verified from cache for $(Split-Path -Leaf $Destination)"
        return
    }

    if ($MS.DryRun) {
        Write-MSLog -Level 'ACTION' -Message "[dry-run] download $Uri"
        return
    }

    $destDir = Split-Path -Parent $Destination
    if (-not (Test-Path $destDir)) { New-Item -ItemType Directory -Path $destDir -Force | Out-Null }

    $shaTmp = "$shaPath.tmp"
    $tmp = "$Destination.tmp"

    Write-MSLog -Level 'ACTION' -Message "Fetching checksum: $ChecksumUri"
    Invoke-WebRequest -Uri $ChecksumUri -OutFile $shaTmp -UseBasicParsing -ErrorAction Stop
    $expected = ((Get-Content -LiteralPath $shaTmp | Select-Object -First 1).Split(' ')[0]).Trim()
    if (-not $expected) {
        Remove-Item -LiteralPath $shaTmp -Force -ErrorAction SilentlyContinue
        throw "Checksum file empty for $ChecksumUri"
    }

    if (Test-Path $Destination -PathType Leaf) {
        $existing = (Get-FileHash -Algorithm SHA256 -LiteralPath $Destination).Hash.ToLowerInvariant()
        if ($existing -eq $expected.ToLowerInvariant()) {
            Move-Item -LiteralPath $shaTmp -Destination $shaPath -Force
            Write-MSLog -Level 'INFO' -Message "Checksum already satisfied for $(Split-Path -Leaf $Destination)"
            return
        }
    }

    Write-MSLog -Level 'ACTION' -Message "Downloading $Uri"
    Invoke-WebRequest -Uri $Uri -OutFile $tmp -UseBasicParsing -ErrorAction Stop
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $tmp).Hash.ToLowerInvariant()
    if ($actual -ne $expected.ToLowerInvariant()) {
        Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $shaTmp -Force -ErrorAction SilentlyContinue
        throw "Checksum verification failed for $Uri"
    }

    Move-Item -LiteralPath $tmp -Destination $Destination -Force
    Move-Item -LiteralPath $shaTmp -Destination $shaPath -Force
}
