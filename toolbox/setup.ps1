# Toolbox Setup Script for Windows
# This script sets up the AI Development Toolbox for use on Windows

param(
    [Parameter(Mandatory=$false)]
    [switch]$SkipExecutionPolicy,
    
    [Parameter(Mandatory=$false)]
    [switch]$InstallGitHooks
)

Write-Host ""
Write-Host "╔═══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   Solana Insurance Toolbox Setup (Windows)       ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"
$SetupErrors = 0

# Check 1: Execution Policy
Write-Host "1. Checking PowerShell execution policy..." -ForegroundColor Yellow
$currentPolicy = Get-ExecutionPolicy -Scope CurrentUser

if ($currentPolicy -eq "Restricted" -or $currentPolicy -eq "AllSigned") {
    Write-Host "   ⚠️  Current policy: $currentPolicy" -ForegroundColor Yellow
    
    if (-not $SkipExecutionPolicy) {
        Write-Host "   💡 PowerShell scripts are blocked. Attempting to fix..." -ForegroundColor Cyan
        
        try {
            Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
            Write-Host "   ✅ Execution policy set to RemoteSigned for CurrentUser" -ForegroundColor Green
        } catch {
            Write-Host "   ❌ Failed to set execution policy: $_" -ForegroundColor Red
            Write-Host ""
            Write-Host "   👉 Manual fix: Run PowerShell as Administrator and execute:" -ForegroundColor Yellow
            Write-Host "      Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser" -ForegroundColor Gray
            Write-Host ""
            $SetupErrors++
        }
    } else {
        Write-Host "   ⚠️  Skipped (use -SkipExecutionPolicy to skip)" -ForegroundColor Yellow
    }
} else {
    Write-Host "   ✅ Execution policy: $currentPolicy (OK)" -ForegroundColor Green
}

# Check 2: Rust and Cargo
Write-Host ""
Write-Host "2. Checking Rust installation..." -ForegroundColor Yellow
try {
    $rustVersion = cargo --version
    Write-Host "   ✅ $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Cargo not found. Install Rust from https://rustup.rs/" -ForegroundColor Red
    $SetupErrors++
}

# Check 3: Solana CLI
Write-Host ""
Write-Host "3. Checking Solana CLI..." -ForegroundColor Yellow
try {
    $solanaVersion = solana --version
    Write-Host "   ✅ $solanaVersion" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️  Solana CLI not found (required for deployment)" -ForegroundColor Yellow
    Write-Host "      Install from: https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Gray
}

# Check 4: Build the toolbox binaries
Write-Host ""
Write-Host "4. Building toolbox binaries..." -ForegroundColor Yellow

Write-Host "   Building CLI..." -ForegroundColor Gray
try {
    Push-Location "toolbox\cli"
    $output = cargo build --release 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ CLI built successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ CLI build failed" -ForegroundColor Red
        Write-Host $output -ForegroundColor Gray
        $SetupErrors++
    }
    Pop-Location
} catch {
    Write-Host "   ❌ Error building CLI: $_" -ForegroundColor Red
    $SetupErrors++
    Pop-Location
}

Write-Host "   Building MCP server..." -ForegroundColor Gray
try {
    Push-Location "toolbox\mcp-server"
    $output = cargo build --release 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ MCP server built successfully" -ForegroundColor Green
    } else {
        Write-Host "   ❌ MCP server build failed" -ForegroundColor Red
        Write-Host $output -ForegroundColor Gray
        $SetupErrors++
    }
    Pop-Location
} catch {
    Write-Host "   ❌ Error building MCP server: $_" -ForegroundColor Red
    $SetupErrors++
    Pop-Location
}

# Check 5: Create aliases (optional)
Write-Host ""
Write-Host "5. Setting up command aliases..." -ForegroundColor Yellow

$profilePath = $PROFILE
$aliasLine = 'function solana-toolbox { cargo run --manifest-path "' + (Get-Location).Path + '\toolbox\cli\Cargo.toml" --release -- $args }'

if (Test-Path $profilePath) {
    $profileContent = Get-Content $profilePath -Raw
    if ($profileContent -notmatch 'solana-toolbox') {
        Write-Host "   💡 Add this to your PowerShell profile ($profilePath):" -ForegroundColor Cyan
        Write-Host "      $aliasLine" -ForegroundColor Gray
        Write-Host ""
        Write-Host "   Or run: Add-Content `$PROFILE '$aliasLine'" -ForegroundColor Gray
    } else {
        Write-Host "   ✅ Alias already exists in profile" -ForegroundColor Green
    }
} else {
    Write-Host "   💡 Create PowerShell profile and add alias:" -ForegroundColor Cyan
    Write-Host "      New-Item -ItemType File -Path `$PROFILE -Force" -ForegroundColor Gray
    Write-Host "      Add-Content `$PROFILE '$aliasLine'" -ForegroundColor Gray
}

# Check 6: Install git hooks
if ($InstallGitHooks) {
    Write-Host ""
    Write-Host "6. Installing git hooks..." -ForegroundColor Yellow
    
    if (Test-Path ".git") {
        $hookPath = ".git\hooks\pre-commit"
        
        # Create a wrapper script that calls the PowerShell script
        $hookContent = @"
#!/bin/sh
# Git pre-commit hook - calls PowerShell script
powershell.exe -ExecutionPolicy Bypass -File toolbox/pipelines/pre-commit.ps1
"@
        
        try {
            $hookContent | Out-File -FilePath $hookPath -Encoding ASCII -NoNewline
            Write-Host "   ✅ Pre-commit hook installed" -ForegroundColor Green
            Write-Host "      Location: $hookPath" -ForegroundColor Gray
        } catch {
            Write-Host "   ❌ Failed to install hook: $_" -ForegroundColor Red
            $SetupErrors++
        }
    } else {
        Write-Host "   ⚠️  Not a git repository" -ForegroundColor Yellow
    }
}

# Summary
Write-Host ""
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

if ($SetupErrors -eq 0) {
    Write-Host "✅ Setup complete! The toolbox is ready to use." -ForegroundColor Green
    Write-Host ""
    Write-Host "📚 Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Read: toolbox\QUICKSTART.md" -ForegroundColor Gray
    Write-Host "  2. Try:  cd toolbox\cli; cargo run -- --help" -ForegroundColor Gray
    Write-Host "  3. Test: .\toolbox\scripts\validate_architecture.ps1" -ForegroundColor Gray
    Write-Host ""
    Write-Host "🎯 Quick commands:" -ForegroundColor Yellow
    Write-Host "  # Scaffold instruction:" -ForegroundColor Gray
    Write-Host "  .\toolbox\scripts\new_instruction.ps1 -Name PayPension -Fields @('amount:u64')" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  # Validate architecture:" -ForegroundColor Gray
    Write-Host "  .\toolbox\scripts\validate_architecture.ps1" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  # Use CLI:" -ForegroundColor Gray
    Write-Host "  cargo run --manifest-path toolbox\cli\Cargo.toml -- new instruction PayPension" -ForegroundColor Cyan
    Write-Host ""
} else {
    Write-Host "⚠️  Setup completed with $SetupErrors error(s)." -ForegroundColor Yellow
    Write-Host "    Please fix the errors above and run setup again." -ForegroundColor Gray
    Write-Host ""
    exit 1
}

Write-Host "🚀 Happy coding with AI assistance!" -ForegroundColor Green
Write-Host ""
