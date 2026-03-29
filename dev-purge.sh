echo "⚠️  WARNING: This will perform a GLOBAL purge from $(pwd)"
read -p "Are you sure? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

if command -v cargo &> /dev/null; then
    echo "  Cleaning Rust..."
    rm -rf "$HOME/.cargo/registry/cache/*"
    rm -rf "$HOME/.cargo/index/*"
fi

if command -v go &> /dev/null; then
    echo "  Cleaning Go..."
    go clean -modcache
fi

if command -v docker &> /dev/null; then
    echo "  Cleaning Docker..."
    docker system prune -a -f
fi

echo "  Cleaning project folders (node_modules, target, .next)..."
find . -type d \( -name "target" -o -name "node_modules" -o -name ".next" -o -name "__pycache__" \) -prune -exec rm -rf {} +

echo "✨ Done!"




