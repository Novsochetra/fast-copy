#!/bin/bash
set -e

# ----------------------
# Config
# ----------------------
SRC="/Users/sochetra.nov/Documents/workspace/personal/emenu"       # Source folder
DST_BASE="/Users/sochetra.nov/Documents/workspace/personal/emenu-backup"   # Base destination
DST_RUST="${DST_BASE}_rust"
DST_CP="${DST_BASE}_cp"
RUST_BINARY="./target/release/fast-copy"     # Adjust if your binary name is different

# ----------------------
# Build Rust release
# ----------------------
echo "🚀 Building Rust release..."
cargo build --release
echo ""

# ----------------------
# Clean previous backups
# ----------------------
echo "🧹 Cleaning old backups..."
rm -rf "$DST_RUST" "$DST_CP"
echo ""

# ----------------------
# Run Rust backup
# ----------------------
echo "📦 Running Rust backup..."
START_RUST=$(date +%s.%N)
"$RUST_BINARY" "$SRC" "$DST_RUST"
END_RUST=$(date +%s.%N)
DURATION_RUST=$(echo "$END_RUST - $START_RUST" | bc)
echo "⏱ Rust backup took: $DURATION_RUST seconds"
echo ""

# ----------------------
# Run cp backup
# ----------------------
echo "📦 Running system cp backup..."
START_CP=$(date +%s.%N)
cp -rp "$SRC" "$DST_CP"
END_CP=$(date +%s.%N)
DURATION_CP=$(echo "$END_CP - $START_CP" | bc)
echo "⏱ cp backup took: $DURATION_CP seconds"
echo ""

# ----------------------
# Compare speed
# ----------------------
if (( $(echo "$DURATION_RUST < $DURATION_CP" | bc -l) )); then
    SPEEDUP=$(echo "$DURATION_CP / $DURATION_RUST" | bc -l)
    echo "⚡ fast-copy is approximately ${SPEEDUP}x faster than system cp"
else
    SPEEDUP=$(echo "$DURATION_RUST / $DURATION_CP" | bc -l)
    echo "⚡ system cp is approximately ${SPEEDUP}x faster than fast-copy"
fi
echo ""

# ----------------------
# Optional: check folder sizes
# ----------------------
echo "📂 Backup sizes:"
du -sh "$DST_RUST" "$DST_CP"
