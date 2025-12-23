#!/bin/bash
set -e

# Configuration
REPO_OWNER="DennySORA"
REPO_NAME="Tool-Package"
BIN_NAME="ops-tools"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS and Arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_TYPE="linux"
        ;;
    Darwin)
        OS_TYPE="macos"
        ;;
    MINGW*|MSYS*|CYGWIN*) 
        OS_TYPE="windows"
        echo "Windows installer is not supported via shell script directly yet. Please download the zip from releases."
        exit 1
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_TYPE="x86_64"
        ;;
    arm64|aarch64)
        if [ "$OS_TYPE" = "macos" ]; then
            ARCH_TYPE="arm64"
        else
            echo "Unsupported Architecture for Linux: $ARCH (Currently only x86_64 is built)"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported Architecture: $ARCH"
        exit 1
        ;;
esac

# Determine Asset Name
ASSET_NAME="${BIN_NAME}-${OS_TYPE}-${ARCH_TYPE}.tar.gz"

echo "Detected: $OS_TYPE $ARCH_TYPE"
echo "Target Asset: $ASSET_NAME"

# Get Latest Release URL
echo "Fetching latest release..."
LATEST_URL=$(curl -s "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest" | grep "browser_download_url" | grep "$ASSET_NAME" | cut -d '"' -f 4)

if [ -z "$LATEST_URL" ]; then
    echo "Error: Could not find a release asset for $ASSET_NAME"
    echo "Please check https://github.com/$REPO_OWNER/$REPO_NAME/releases"
    exit 1
fi

echo "Downloading from $LATEST_URL..."
TEMP_DIR=$(mktemp -d)
curl -L -o "$TEMP_DIR/$ASSET_NAME" "$LATEST_URL"

# Extract
echo "Extracting..."
tar -xzf "$TEMP_DIR/$ASSET_NAME" -C "$TEMP_DIR"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TEMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

# Cleanup
rm -rf "$TEMP_DIR"

echo "Successfully installed to $INSTALL_DIR/$BIN_NAME"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "WARNING: $INSTALL_DIR is not in your PATH."
    echo "Add the following to your shell config (.bashrc, .zshrc, etc.):"
    echo "  export PATH=\"
$PATH:$INSTALL_DIR\""
fi

echo ""
echo "Run '$BIN_NAME' to start!"
