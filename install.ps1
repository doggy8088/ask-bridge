# install.ps1 for Windows PowerShell

$ErrorActionPreference = "Stop"

Write-Host "Starting Ask Bridge installation for Windows..." -ForegroundColor Cyan

# 1. Check Node.js and npx
$nodeCheck = Get-Command node -ErrorAction SilentlyContinue
$npxCheck = Get-Command npx -ErrorAction SilentlyContinue

if (-not $nodeCheck) {
    Write-Error "Node.js is not installed. Please install Node.js (https://nodejs.org/) and retry."
    exit 1
}

if (-not $npxCheck) {
    Write-Error "npx is not installed. Please ensure NPM/npx is available in your PATH."
    exit 1
}

$nodeVersionOutput = & node --version 2>&1
$nodeVersionExitCode = $LASTEXITCODE
$nodeVersionText = ($nodeVersionOutput | Out-String).Trim()

if ($nodeVersionExitCode -ne 0 -or $nodeVersionText -notmatch '^v?(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$') {
    Write-Error "Could not determine a supported Node.js version. Install a current Node.js LTS release, reopen PowerShell, and retry."
    exit 1
}

$nodeMajor = [int]$Matches[1]
$nodeMinor = [int]$Matches[2]
$nodePatch = [int]$Matches[3]
$nodeVersionSupported = `
    ($nodeMajor -eq 20 -and ($nodeMinor -gt 19 -or ($nodeMinor -eq 19 -and $nodePatch -ge 0))) -or `
    ($nodeMajor -eq 22 -and ($nodeMinor -gt 12 -or ($nodeMinor -eq 12 -and $nodePatch -ge 0))) -or `
    ($nodeMajor -ge 23)

if (-not $nodeVersionSupported) {
    Write-Error "Node.js $nodeVersionText is not supported by chrome-devtools-mcp@latest. Supported versions are ^20.19.0, ^22.12.0, or >=23.0.0. Install a current Node.js LTS release, reopen PowerShell, and retry."
    exit 1
}

# 2. Check Google Chrome
$chromePaths = @(
    "$env:ProgramFiles\Google\Chrome\Application\chrome.exe",
    "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe",
    "$env:LocalAppData\Google\Chrome\Application\chrome.exe"
)

$chromeFound = $false
foreach ($path in $chromePaths) {
    if (Test-Path $path) {
        $chromeFound = $true
        break
    }
}

if (-not $chromeFound) {
    Write-Host "Warning: Google Chrome was not found in default installation paths." -ForegroundColor Yellow
    Write-Host "Please ensure Google Chrome is installed, as it is required by Chrome DevTools MCP." -ForegroundColor Yellow
}

# 3. Target configuration
$Version = "0.2.2"
$RepoOwner = "doggy8088"
$RepoName = "ask-bridge"
$ArtifactName = "ask-bridge-x86_64-pc-windows-msvc.zip"
$ReleaseUrl = "https://github.com/$RepoOwner/$RepoName/releases/download/v$Version/$ArtifactName"

# 4. Create installation directory
$InstallDir = Join-Path $HOME ".local\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$TempDir = Join-Path $env:TEMP "ask-bridge-install"
if (Test-Path $TempDir) {
    Remove-Item -Recurse -Force $TempDir
}
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    # 5. Download zip
    Write-Host "Downloading $ArtifactName..." -ForegroundColor Cyan
    $ZipPath = Join-Path $TempDir $ArtifactName
    Invoke-WebRequest -Uri $ReleaseUrl -OutFile $ZipPath

    # 6. Extract zip
    Write-Host "Extracting archive..." -ForegroundColor Cyan
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    # Find the executable
    $ExePath = Get-ChildItem -Path $TempDir -Recurse -Filter "ask-bridge.exe" | Select-Object -First 1
    if (-not $ExePath) {
        Write-Error "Could not find ask-bridge.exe in the downloaded archive."
        exit 1
    }

    # Copy to destination as ask-bridge.exe and keep ask.exe as an alias.
    $DestPath = Join-Path $InstallDir "ask-bridge.exe"
    $AliasPath = Join-Path $InstallDir "ask.exe"
    Write-Host "Installing ask-bridge.exe to $InstallDir..." -ForegroundColor Cyan
    Copy-Item -Path $ExePath.FullName -Destination $DestPath -Force
    Copy-Item -Path $ExePath.FullName -Destination $AliasPath -Force
}
finally {
    # Clean up temp
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir
    }
}

# 7. Check/Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$CleanPathList = $UserPath -split ';'

if ($CleanPathList -notcontains $InstallDir) {
    Write-Host "Adding $InstallDir to User PATH..." -ForegroundColor Cyan
    $NewPath = $UserPath
    if ($NewPath -and -not $NewPath.EndsWith(';')) {
        $NewPath += ";"
    }
    $NewPath += $InstallDir
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    
    # Update current session path
    $env:Path = $env:Path + ";" + $InstallDir
    Write-Host "Successfully added to PATH. You may need to restart your terminal to apply." -ForegroundColor Green
}

Write-Host "Successfully installed! You can now use the 'ask-bridge' command. The 'ask' alias is also available." -ForegroundColor Green
