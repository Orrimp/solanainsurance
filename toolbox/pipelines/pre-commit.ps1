# PowerShell pre-commit validation hook
$ErrorActionPreference = "Stop"

Write-Host "🔍 Running pre-commit checks..." -ForegroundColor Cyan

$Failed = $false

# 1. Cargo check
Write-Host "1️⃣  Running cargo check..." -ForegroundColor Yellow
if ((cargo check 2>&1); $LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Passed" -ForegroundColor Green
} else {
    Write-Host "   ❌ Failed" -ForegroundColor Red
    $Failed = $true
}

# 2. Clippy (warnings only)
Write-Host "2️⃣  Running clippy..." -ForegroundColor Yellow
cargo clippy --lib -- -D warnings 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Passed" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  Warnings found (not blocking)" -ForegroundColor Yellow
}

# 3. Architecture validation (native CLI)
Write-Host "3️⃣  Validating architecture..." -ForegroundColor Yellow
$toolbox = ".\target\release\solana-toolbox.exe"
if (Test-Path $toolbox) {
    & $toolbox validate all
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Passed" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Failed" -ForegroundColor Red
        $Failed = $true
    }
} else {
    Write-Host "   ⚠️  Toolbox not built, skipping" -ForegroundColor Yellow
}
}

# 4. Formatting
Write-Host "4️⃣  Checking code formatting..." -ForegroundColor Yellow
cargo fmt -- --check 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Formatted" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  Auto-formatting..." -ForegroundColor Yellow
    cargo fmt
    Write-Host "   ✅ Done" -ForegroundColor Green
}

Write-Host ""
if ($Failed) {
    Write-Host "❌ Pre-commit checks failed!" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ All checks passed!" -ForegroundColor Green
    exit 0
}
