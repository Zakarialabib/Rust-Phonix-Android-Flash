#!/bin/bash
# Phoenix Tools Bootstrap Script for Linux/macOS
# Sets up the development environment and builds all tools

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo ""
echo -e "${CYAN}  ╔═══════════════════════════════════════╗${NC}"
echo -e "${CYAN}  ║       Phoenix Tools Bootstrap         ║${NC}"
echo -e "${CYAN}  ╚═══════════════════════════════════════╝${NC}"
echo ""

BUILD_MODE="debug"
BUILD_GUI=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release) BUILD_MODE="release"; shift ;;
        --gui) BUILD_GUI=true; shift ;;
        --help)
            echo "Usage: bootstrap.sh [--release] [--gui] [--help]"
            echo ""
            echo "Options:"
            echo "  --release   Build in release mode"
            echo "  --gui       Also build the Tauri GUI"
            echo "  --help      Show this help message"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Check for Rust
echo -e "${YELLOW}[1/5] Checking for Rust...${NC}"
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}      Rust not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}      Rust installed successfully!${NC}"
else
    VERSION=$(rustc --version)
    echo -e "${GREEN}      Found: $VERSION${NC}"
fi

# Add ARM64 target
echo -e "${YELLOW}[2/5] Adding ARM64 cross-compile target...${NC}"
rustup target add aarch64-unknown-linux-gnu 2>/dev/null || true
echo -e "${GREEN}      Target added: aarch64-unknown-linux-gnu${NC}"

# Build CLI
echo -e "${YELLOW}[3/5] Building phoenix-cli...${NC}"
cd "$PROJECT_ROOT"

BUILD_FLAG=""
if [ "$BUILD_MODE" = "release" ]; then
    BUILD_FLAG="--release"
fi

cargo build $BUILD_FLAG -p phoenix-cli

if [ $? -ne 0 ]; then
    echo -e "${RED}      Build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}      phoenix-cli built successfully!${NC}"

# Build GUI (optional)
if [ "$BUILD_GUI" = true ]; then
    echo -e "${YELLOW}[4/5] Building phoenix-gui (Tauri)...${NC}"
    
    if ! command -v node &> /dev/null; then
        echo -e "${RED}      Node.js not found. Please install it first.${NC}"
        exit 1
    fi
    
    cd "$PROJECT_ROOT/ui"
    npm install
    
    cd "$PROJECT_ROOT/phoenix-gui"
    cargo build $BUILD_FLAG
    
    echo -e "${GREEN}      phoenix-gui built successfully!${NC}"
else
    echo -e "\033[1;30m[4/5] Skipping GUI build (use --gui to include)${NC}"
fi

# Install to user PATH
echo -e "${YELLOW}[5/5] Installing to user PATH...${NC}"
INSTALL_PATH="$HOME/.phoenix/bin"
mkdir -p "$INSTALL_PATH"

TARGET_DIR="$BUILD_MODE"
cp "$PROJECT_ROOT/target/$TARGET_DIR/phoenix" "$INSTALL_PATH/phoenix"
chmod +x "$INSTALL_PATH/phoenix"

# Add to PATH in shell config
SHELL_RC="$HOME/.bashrc"
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
fi

if ! grep -q ".phoenix/bin" "$SHELL_RC" 2>/dev/null; then
    echo 'export PATH="$HOME/.phoenix/bin:$PATH"' >> "$SHELL_RC"
    echo -e "${GREEN}      Added $INSTALL_PATH to PATH in $SHELL_RC${NC}"
fi

echo ""
echo -e "${GREEN}  ✅ Bootstrap complete!${NC}"
echo ""
echo -e "${CYAN}  Usage:${NC}"
echo "    phoenix detect          # Scan for connected devices"
echo "    phoenix config init     # Create device configuration"
echo "    phoenix build           # Build firmware image"
echo "    phoenix flash           # Write image to device"
echo ""
echo "  Note: Run 'source $SHELL_RC' or restart your terminal."
echo ""
