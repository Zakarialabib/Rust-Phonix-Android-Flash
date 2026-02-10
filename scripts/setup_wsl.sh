#!/bin/bash

# Phoenix Tools - WSL Setup Script
# Installs required dependencies for building Android TV box images

set -e

echo "🔥 Phoenix Tools - WSL Setup"
echo "============================"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (sudo)"
  exit 1
fi

echo "📦 Updating package lists..."
apt-get update

echo "📦 Installing build dependencies..."
# Core build tools
apt-get install -y build-essential git make gcc g++ unzip bc bison flex libssl-dev

# Device Tree Compiler
apt-get install -y device-tree-compiler

# U-Boot tools (mkimage)
apt-get install -y u-boot-tools

# Python (for build scripts)
apt-get install -y python3 python3-pip

# Compression tools
apt-get install -y lz4 xz-utils

# Library dependencies
apt-get install -y libncurses5-dev libncursesw5-dev

echo "✅ Dependencies installed successfully!"
echo "You can now use the Phoenix build pipeline from Windows."
