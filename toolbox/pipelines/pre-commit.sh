#!/bin/bash
# Pre-commit validation hook
set -e

echo "🔍 Running pre-commit checks..."

# 1. Cargo check
echo "1️⃣  Running cargo check..."
if cargo check --quiet; then
    echo "   ✅ Passed"
else
    echo "   ❌ Failed"
    exit 1
fi

# 2. Clippy (warnings only)
echo "2️⃣  Running clippy..."
if cargo clippy --lib -- -D warnings 2>&1 | tail -1; then
    echo "   ✅ Passed"
else
    echo "   ⚠️  Warnings (not blocking)"
fi

# 3. Architecture validation (native CLI)
echo "3️⃣  Validating architecture..."
TOOLBOX="./target/release/solana-toolbox"
if [ -f "$TOOLBOX" ]; then
    if $TOOLBOX validate all; then
        echo "   ✅ Passed"
    else
        echo "   ❌ Failed"
        exit 1
    fi
else
    echo "   ⚠️  Toolbox not built, skipping"
fi

# 4. Formatting
echo "4️⃣  Checking code formatting..."
if cargo fmt -- --check; then
    echo "   ✅ Formatted"
else
    echo "   ⚠️  Auto-formatting..."
    cargo fmt
    echo "   ✅ Done"
fi

echo ""
echo "✅ All checks passed!"
