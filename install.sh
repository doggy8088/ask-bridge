#!/bin/bash
set -e

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}Starting Ask Bridge installation...${NC}"

# 1. Check Node.js and npx
if ! command -v node >/dev/null 2>&1; then
    echo -e "${RED}Error: Node.js is not installed.${NC}"
    echo -e "${YELLOW}Please install Node.js (https://nodejs.org/) and retry.${NC}"
    exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
    echo -e "${RED}Error: npx is not installed.${NC}"
    echo -e "${YELLOW}Please make sure NPM/npx is available in your PATH.${NC}"
    exit 1
fi

# 2. Check Google Chrome
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" = "Darwin" ]; then
    if [ ! -x "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" ]; then
        echo -e "${YELLOW}Warning: Google Chrome was not found at /Applications/Google Chrome.app.${NC}"
        if command -v brew >/dev/null 2>&1; then
            echo -e "${CYAN}Installing Google Chrome via Homebrew...${NC}"
            brew install --cask google-chrome
        else
            echo -e "${YELLOW}Please install Google Chrome manually: https://www.google.com/chrome/${NC}"
        fi
    fi
elif [ "$OS" = "Linux" ]; then
    if ! command -v google-chrome >/dev/null 2>&1 && ! command -v google-chrome-stable >/dev/null 2>&1; then
        echo -e "${YELLOW}Warning: Google Chrome was not found in your PATH.${NC}"
        echo -e "${YELLOW}Please make sure Google Chrome is installed, as it is required by Chrome DevTools MCP.${NC}"
    fi
fi

# 3. Determine target architecture and file name
VERSION="0.1.1"
REPO_OWNER="doggy8088"
REPO_NAME="ask-bridge"

if [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        TARGET="aarch64-apple-darwin"
    else
        TARGET="x86_64-apple-darwin"
    fi
    EXT="tar.xz"
elif [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET="x86_64-unknown-linux-gnu"
    else
        echo -e "${RED}Error: Unsupported Linux architecture: $ARCH. Only x86_64 is supported.${NC}"
        exit 1
    fi
    EXT="tar.xz"
else
    echo -e "${RED}Error: Unsupported operating system: $OS. For Windows, please run install.ps1.${NC}"
    exit 1
fi

ARTIFACT_NAME="ask-bridge-${TARGET}.${EXT}"
RELEASE_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/v${VERSION}/${ARTIFACT_NAME}"

# 4. Create installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

echo -e "${CYAN}Downloading ${ARTIFACT_NAME}...${NC}"
if command -v curl >/dev/null 2>&1; then
    curl -L "$RELEASE_URL" -o "$TEMP_DIR/$ARTIFACT_NAME"
elif command -v wget >/dev/null 2>&1; then
    wget "$RELEASE_URL" -O "$TEMP_DIR/$ARTIFACT_NAME"
else
    echo -e "${RED}Error: Neither curl nor wget was found. Please install one of them to proceed.${NC}"
    exit 1
fi

echo -e "${CYAN}Extracting archive...${NC}"
tar -xJf "$TEMP_DIR/$ARTIFACT_NAME" -C "$TEMP_DIR"

# Find extracted binary (it could be in a subdirectory or direct)
BINARY_PATH=$(find "$TEMP_DIR" -type f -name "ask-bridge" | head -n 1)

if [ -z "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Could not find ask-bridge binary in the downloaded archive.${NC}"
    exit 1
fi

# 5. Install binary
echo -e "${CYAN}Installing ask to $INSTALL_DIR/ask...${NC}"
cp "$BINARY_PATH" "$INSTALL_DIR/ask"
chmod +x "$INSTALL_DIR/ask"

# 6. Check PATH
case :$PATH: in
    *:$INSTALL_DIR:*) ;;
    *) 
        echo -e "${YELLOW}Warning: $INSTALL_DIR is not in your PATH.${NC}"
        echo -e "${YELLOW}To run 'ask' globally, add it to your shell configuration (e.g. ~/.bashrc, ~/.zshrc):${NC}"
        echo -e "${CYAN}  export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        ;;
esac

echo -e "${GREEN}Successfully installed! You can now use the 'ask' command.${NC}"
