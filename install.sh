#!/bin/bash
# ============================================================================
# DEV-PURGE Installation Script
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_PATH="$HOME/.local/bin"

echo "🛡️  DEV-PURGE Installer"
echo "====================="
echo

# Check if dev-purge.sh exists
if [[ ! -f "$SCRIPT_DIR/dev-purge.sh" ]]; then
    echo "❌ Error: dev-purge.sh not found in $SCRIPT_DIR"
    exit 1
fi

# Create installation directory if needed
if [[ ! -d "$INSTALL_PATH" ]]; then
    echo "📁 Creating $INSTALL_PATH..."
    mkdir -p "$INSTALL_PATH"
fi

# Copy the script
echo "📋 Installing dev-purge to $INSTALL_PATH..."
cp "$SCRIPT_DIR/dev-purge.sh" "$INSTALL_PATH/dev-purge"
chmod +x "$INSTALL_PATH/dev-purge"

# Add to PATH if not already there
if ! grep -q "$INSTALL_PATH" "$HOME/.bashrc" 2>/dev/null; then
    echo "🔗 Adding $INSTALL_PATH to \$PATH in ~/.bashrc..."
    echo "" >> "$HOME/.bashrc"
    echo "# Dev-Purge Installation" >> "$HOME/.bashrc"
    echo "export PATH=\"\$PATH:$INSTALL_PATH\"" >> "$HOME/.bashrc"
fi

echo
echo "✅ Installation Complete!"
echo
echo "Usage:"
echo "  dev-purge --dry-run    # Preview what will be cleaned"
echo "  dev-purge              # Actually clean up"
echo
echo "To use the command immediately, run:"
echo "  source ~/.bashrc"
echo
echo "Or just open a new terminal and type: dev-purge"
