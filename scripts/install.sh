#!/bin/bash
set -e

# oops installer script for Unix systems

VERSION="${OOPS_VERSION:-latest}"
INSTALL_DIR="${OOPS_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux*)
        OS="linux"
        ;;
    darwin*)
        OS="darwin"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

BINARY_NAME="oops-${OS}-${ARCH}"

echo "Installing oops for ${OS}/${ARCH}..."

# Get download URL
if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/oops-cli/oops/releases/latest/download/${BINARY_NAME}"
else
    DOWNLOAD_URL="https://github.com/oops-cli/oops/releases/download/${VERSION}/${BINARY_NAME}"
fi

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download
echo "Downloading from ${DOWNLOAD_URL}..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/oops"

# Make executable
chmod +x "$TMP_DIR/oops"

# Install
echo "Installing to ${INSTALL_DIR}/oops..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/oops" "$INSTALL_DIR/oops"
else
    sudo mv "$TMP_DIR/oops" "$INSTALL_DIR/oops"
fi

echo ""
echo "oops installed successfully!"
echo ""
echo "To complete setup, add to your shell config:"
echo ""
echo "  # Bash (~/.bashrc)"
echo '  eval "$(oops --alias)"'
echo ""
echo "  # Zsh (~/.zshrc)"
echo '  eval "$(oops --alias)"'
echo ""
echo "  # Fish (~/.config/fish/config.fish)"
echo "  oops --alias | source"
echo ""
echo "Then restart your shell or run: source ~/.bashrc"
