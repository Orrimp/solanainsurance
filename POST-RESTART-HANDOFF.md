# Post Restart Handoff for Copilot

Use this file after Windows restart to continue troubleshooting quickly.

## 1) What I already changed
- Improved validator startup diagnostics in toolbox/cli/src/deployment.rs.
- Added Windows-specific guidance when validator fails with Access is denied (os error 5).

## 2) Run these commands after restart
Run in PowerShell from repo root.

### A. Symlink capability check
$target = Join-Path $env:TEMP 'symlink-target-test.txt'; $link = Join-Path $env:TEMP 'symlink-link-test.txt'; Set-Content -Path $target -Value 'x'; if (Test-Path $link) { Remove-Item $link -Force }; try { New-Item -ItemType SymbolicLink -Path $link -Target $target -ErrorAction Stop | Out-Null; Write-Host 'SYMLINK_TEST: PASS' } catch { Write-Host ('SYMLINK_TEST: FAIL - ' + $_.Exception.Message) } finally { if (Test-Path $link) { Remove-Item $link -Force }; if (Test-Path $target) { Remove-Item $target -Force } }

### B. Validator direct test
$ledger = Join-Path $env:TEMP 'solana-restart-test-ledger'; if (Test-Path $ledger) { Remove-Item -Recurse -Force $ledger }; solana-test-validator --reset --ledger $ledger

### C. Pipeline deploy test
cargo run --release --manifest-path toolbox/cli/Cargo.toml -- pipeline --deploy --validator-timeout 30

## 3) Paste results for Copilot
Copy this block into chat and fill it.

Restart done: YES/NO
Opened terminal as admin: YES/NO

Symlink test result:
<PASTE OUTPUT>

Validator direct test result:
<PASTE OUTPUT>

Pipeline deploy test result:
<PASTE OUTPUT>

Any extra error text from target/validator.log:
<PASTE OUTPUT>

## 4) If still failing
Try once in Administrator terminal and run section 2B and 2C again.
