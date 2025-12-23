#!/bin/bash
set -e

REPO="DennySORA/Tool-Package"
BINARY_NAME="ops-tools"
INSTALL_DIR="/usr/local/bin"

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
        if [ "$OS" = "Darwin" ]; then
            ARCH_TYPE="arm64"
        else
            echo "Unsupported Architecture: $ARCH on Linux (only x86_64 supported for now)"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported Architecture: $ARCH"
        exit 1
        ;;
esac

ASSET_NAME="${BINARY_NAME}-${OS_TYPE}-${ARCH_TYPE}.tar.gz"

echo "Detected platform: $OS_TYPE $ARCH_TYPE"
echo "Looking for asset: $ASSET_NAME"

# Get latest release URL
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
echo "Fetching latest release info..."
RELEASE_DATA=$(curl -s $LATEST_RELEASE_URL)
DOWNLOAD_URL=$(echo "$RELEASE_DATA" | grep "browser_download_url" | grep "$ASSET_NAME" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: Could not find download URL for $ASSET_NAME in latest release."
    echo "Please check if a release exists at https://github.com/$REPO/releases"
    exit 1
fi

echo "Downloading from: $DOWNLOAD_URL"
curl -L -o "/tmp/$ASSET_NAME" "$DOWNLOAD_URL"

echo "Extracting..."
tar -xzf "/tmp/$ASSET_NAME" -C /tmp/

echo "Installing to $INSTALL_DIR (requires sudo)..."
if command -v sudo >/dev/null 2>&1; then
    sudo mv "/tmp/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
else
    mv "/tmp/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
fi

rm "/tmp/$ASSET_NAME"

echo "Installation complete! Try running '$BINARY_NAME --help'"