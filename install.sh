#!/bin/bash

# tl installer script
# Usage: curl -sSL https://raw.githubusercontent.com/eyalev/tl/master/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Emojis
PACKAGE="📦"
CHECK="✅"
WARNING="⚠️"
DOWNLOAD="⬇️"
ROCKET="🚀"

echo -e "${PACKAGE} Installing tl - Tool Installer"
echo

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "${OS}" in
    linux*)
        OS="linux"
        ;;
    darwin*)
        OS="darwin"
        ;;
    *)
        echo -e "${RED}❌ Unsupported OS: ${OS}${NC}"
        exit 1
        ;;
esac

case "${ARCH}" in
    x86_64|amd64)
        ARCH="amd64"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: ${ARCH}${NC}"
        exit 1
        ;;
esac

PLATFORM="${OS}-${ARCH}"
BINARY_NAME="tl-${PLATFORM}"
if [ "${OS}" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

echo -e "${BLUE}🔍 Detected platform: ${PLATFORM}${NC}"

# Get latest release information
echo -e "${BLUE}🔍 Fetching latest release information...${NC}"
RELEASE_URL="https://api.github.com/repos/eyalev/tl/releases/latest"
DOWNLOAD_URL=$(curl -s "${RELEASE_URL}" | grep "browser_download_url.*${BINARY_NAME}" | cut -d '"' -f 4)

if [ -z "${DOWNLOAD_URL}" ]; then
    echo -e "${RED}❌ Could not find binary for platform: ${PLATFORM}${NC}"
    echo "Available binaries:"
    curl -s "${RELEASE_URL}" | grep "browser_download_url" | cut -d '"' -f 4 | sed 's|.*/||'
    exit 1
fi

echo -e "${BLUE}${DOWNLOAD} Downloading from: ${DOWNLOAD_URL}${NC}"

# Create install directory
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

# Download and install
TEMP_FILE=$(mktemp)
trap 'rm -f "${TEMP_FILE}"' EXIT

if command -v curl >/dev/null 2>&1; then
    curl -L -o "${TEMP_FILE}" "${DOWNLOAD_URL}"
elif command -v wget >/dev/null 2>&1; then
    wget -O "${TEMP_FILE}" "${DOWNLOAD_URL}"
else
    echo -e "${RED}❌ Neither curl nor wget is available${NC}"
    exit 1
fi

# Install binary
INSTALL_PATH="${INSTALL_DIR}/tl"
mv "${TEMP_FILE}" "${INSTALL_PATH}"
chmod +x "${INSTALL_PATH}"

echo -e "${CHECK} Successfully installed tl to ${INSTALL_PATH}"

# Check if ~/.local/bin is in PATH
if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
    echo
    echo -e "${WARNING} ${INSTALL_DIR} is not in your PATH"
    echo "To use tl from anywhere, add this to your shell profile:"
    echo
    echo -e "${YELLOW}  export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
    echo
    echo "For bash users:"
    echo -e "${YELLOW}  echo 'export PATH=\"\$PATH:${INSTALL_DIR}\"' >> ~/.bashrc${NC}"
    echo -e "${YELLOW}  source ~/.bashrc${NC}"
    echo
    echo "For zsh users:"
    echo -e "${YELLOW}  echo 'export PATH=\"\$PATH:${INSTALL_DIR}\"' >> ~/.zshrc${NC}"
    echo -e "${YELLOW}  source ~/.zshrc${NC}"
    echo
fi

echo -e "${ROCKET} Installation complete!"
echo
echo "Try it out:"
echo -e "${BLUE}  tl list${NC}      # List available tools"
echo -e "${BLUE}  tl install <tool>${NC}  # Install a tool"
echo -e "${BLUE}  tl --help${NC}    # Show help"

# Test installation
if command -v tl >/dev/null 2>&1; then
    echo
    echo "🎉 tl is ready to use!"
    tl --help
else
    echo
    echo -e "${WARNING} tl command not found in PATH. You may need to restart your shell or run:"
    echo -e "${YELLOW}  export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
fi