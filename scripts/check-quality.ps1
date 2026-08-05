# Visual Library — quality gate (Windows)
# Usage: pwsh -File scripts/check-quality.ps1 [-Strict] [-SkipTests]
# Maps to: docs/reglas-calidad-codigo.md

param(
    [switch]$Strict,
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

function Step($name, $scriptBlock) {
    Write-Host ""
    Write-Host "==> $name" -ForegroundColor Cyan
    & $scriptBlock
    if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
        throw "FAILED: $name (exit $LASTEXITCODE)"
    }
}

Write-Host "Visual Library quality check (root: $root)" -ForegroundColor Green
if ($Strict) { Write-Host "Mode: STRICT (clippy -D warnings)" -ForegroundColor Yellow }

Step "cargo fmt --check" {
    cargo fmt --all -- --check
}

Step "cargo clippy" {
    if ($Strict) {
        cargo clippy --workspace --all-targets -- -D warnings
    } else {
        # Warnings allowed until legacy is cleaned; still fails on hard errors.
        cargo clippy --workspace --all-targets -- -W clippy::correctness
    }
}

if (-not $SkipTests) {
    Step "cargo test --workspace" {
        cargo test --workspace
    }
} else {
    Write-Host "==> skip cargo test" -ForegroundColor DarkGray
}

Step "pnpm ui: tsc + vitest" {
    pnpm --filter @visual-library/ui exec tsc --noEmit
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    if (-not $SkipTests) {
        pnpm --filter @visual-library/ui test
    }
}

Write-Host ""
Write-Host "Quality check OK." -ForegroundColor Green
Write-Host "Rules: docs/reglas-calidad-codigo.md"
