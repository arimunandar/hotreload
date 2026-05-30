#!/bin/bash
set -euo pipefail

REPO="arimunandar/hotreload"
INSTALL_DIR="${HOME}/.local/bin"

echo "Installing hotreload..."

# Check for Rust toolchain
if ! command -v cargo &>/dev/null; then
    echo "Error: Rust toolchain not found."
    echo "Install it first: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Clone and build
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "Cloning..."
git clone --depth 1 "https://github.com/${REPO}.git" "$TMPDIR/hotreload" 2>/dev/null

echo "Building (this takes ~30s)..."
cd "$TMPDIR/hotreload"
cargo build --release --quiet

# Install binary
mkdir -p "$INSTALL_DIR"
cp target/release/hotreload "$INSTALL_DIR/hotreload"
chmod +x "$INSTALL_DIR/hotreload"

# Check if install dir is in PATH
if ! echo "$PATH" | tr ':' '\n' | grep -q "^${INSTALL_DIR}$"; then
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        zsh)  RC_FILE="$HOME/.zshrc" ;;
        bash) RC_FILE="$HOME/.bashrc" ;;
        *)    RC_FILE="$HOME/.profile" ;;
    esac

    echo "" >> "$RC_FILE"
    echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$RC_FILE"
    echo "Added ${INSTALL_DIR} to PATH in ${RC_FILE}"
    echo "Run: source ${RC_FILE}"
fi

echo ""
echo "✅ hotreload installed to ${INSTALL_DIR}/hotreload"
echo ""
echo "Next steps:"
echo "  1. Add HotReloadKit to your Xcode project:"
echo "     File → Add Package → https://github.com/arimunandar/HotReloadKit"
echo "  2. Initialize your project:"
echo "     cd your-project && hotreload init"
echo "  3. Start watching:"
echo "     hotreload watch"
