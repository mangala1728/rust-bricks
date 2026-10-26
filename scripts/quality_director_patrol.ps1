# Quality Director Local Scheduled Patrol Script

param (
    [switch]$RegisterTask,
    [switch]$UnregisterTask,
    [switch]$Silent
)

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

if ($RegisterTask) {
    $TaskName = "RustBricks_QualityDirector_NightlyPatrol"
    $Action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$PSScriptRoot\quality_director_patrol.ps1`" -Silent"
    $Trigger = New-ScheduledTaskTrigger -Daily -At "03:00AM"
    $Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
    
    Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Settings $Settings -Description "rust-bricks Zero-Trust Quality Director Daily Patrol" -Force
    Write-Host "[Quality Director] Successfully registered Windows Daily Task: $TaskName" -ForegroundColor Green
    exit 0
}

if ($UnregisterTask) {
    $TaskName = "RustBricks_QualityDirector_NightlyPatrol"
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Write-Host "[Quality Director] Removed Windows Task: $TaskName" -ForegroundColor Yellow
    exit 0
}

$Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Quality Director Patrol: Starting Audit ($Timestamp)" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Cargo test
Write-Host "1. Running Full Unit Test Suite..." -ForegroundColor Yellow
cargo test --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: cargo test failed!" -ForegroundColor Red
    exit 1
}

# 2. Guard checks
Write-Host "2. Running Contract Guard (Line 1-5)..." -ForegroundColor Yellow
cargo run --quiet -- guard
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: brick-guard failed!" -ForegroundColor Red
    exit 1
}

# 3. Chaos Audit
Write-Host "3. Running Zero-Trust Chaos Adversarial Audit..." -ForegroundColor Yellow
cargo run --quiet -- audit
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Chaos audit failed!" -ForegroundColor Red
    exit 1
}

# 4. Remote branch patrol
Write-Host "4. Running Branch Patrol..." -ForegroundColor Yellow
cargo run --quiet -- patrol

# 5. Wishlist scan
Write-Host "5. Scanning Wishlist Board..." -ForegroundColor Yellow
cargo run --quiet -- wish list

Write-Host "`nSUCCESS: Quality Director Patrol Complete! 100% PASS." -ForegroundColor Green
