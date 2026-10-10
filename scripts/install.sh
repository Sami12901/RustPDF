#!/usr/bin/env bash
# RustPDF Linux / macOS Terminal One-Line Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/Sami12901/RustPDF/main/scripts/install.sh | bash

set -e

REPO="Sami12901/RustPDF"
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"

echo -e "\033[1;36m=======================================================\033[0m"
echo -e "\033[1;37m        RustPDF Terminal Installer (Linux / macOS)     \033[0m"
echo -e "\033[1;36m=======================================================\033[0m"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_TAG="linux"
        ;;
    Darwin)
        OS_TAG="macos"
        ;;
    *)
        echo -e "\033[0;31m[ERROR] Unsupported operating system: $OS\033[0m"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_TAG="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TAG="aarch64"
        ;;
    *)
        echo -e "\033[0;31m[ERROR] Unsupported CPU architecture: $ARCH\033[0m"
        exit 1
        ;;
esac

TAR_NAME="rustpdf-${OS_TAG}-${ARCH_TAG}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${TAR_NAME}"

mkdir -p "$INSTALL_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo -e "\033[1;33m-> Downloading ${TAR_NAME} from GitHub Releases...\033[0m"
if curl -sSfL "$DOWNLOAD_URL" -o "$TMP_DIR/rustpdf.tar.gz" 2>/dev/null; then
    echo -e "\033[1;33m-> Extracting to ${INSTALL_DIR}...\033[0m"
    tar -xzf "$TMP_DIR/rustpdf.tar.gz" -C "$TMP_DIR"
    cp -f "$TMP_DIR/rustpdf" "$INSTALL_DIR/rustpdf"
    [ -f "$TMP_DIR/rustpdf-cli" ] && cp -f "$TMP_DIR/rustpdf-cli" "$INSTALL_DIR/rustpdf-cli"
    chmod +x "$INSTALL_DIR/rustpdf" "$INSTALL_DIR/rustpdf-cli" 2>/dev/null || true
else
    echo -e "\033[0;33m-> Release binary not found on GitHub. Checking for Cargo to build from source...\033[0m"
    if command -v cargo >/dev/null 2>&1; then
        echo -e "\033[1;32m-> Building release binaries via Cargo...\033[0m"
        cargo install --git "https://github.com/${REPO}.git" rustpdf rustpdf-cli --root "${HOME}/.local"
    else
        echo -e "\033[0;31m[ERROR] Prebuilt binaries not yet available on GitHub and Cargo is not installed.\033[0m"
        echo "Please install Rust (https://rustup.rs) or visit https://github.com/${REPO}/releases."
        exit 1
    fi
fi

# Create Linux desktop entry
if [ "$OS" = "Linux" ] && [ -d "$HOME/.local/share" ]; then
    mkdir -p "$DESKTOP_DIR"
    cat <<EOF > "$DESKTOP_DIR/rustpdf.desktop"
[Desktop Entry]
Name=RustPDF
Comment=High-Performance Desktop PDF Workbench
Exec=${INSTALL_DIR}/rustpdf %U
Icon=application-pdf
Terminal=false
Type=Application
Categories=Office;Viewer;Graphics;
MimeType=application/pdf;
EOF
    chmod +x "$DESKTOP_DIR/rustpdf.desktop"
    echo -e "\033[1;32m-> Registered Desktop application shortcut: ~/.local/share/applications/rustpdf.desktop\033[0m"
fi

# Ensure PATH includes ~/.local/bin
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_RC=""
    if [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    fi

    if [ -n "$SHELL_RC" ] && [ -f "$SHELL_RC" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
        echo -e "\033[1;32m-> Added $INSTALL_DIR to PATH in $SHELL_RC\033[0m"
    fi
fi

echo -e "\n\033[1;32m[SUCCESS] RustPDF installed successfully to ${INSTALL_DIR}/rustpdf\033[0m"
echo -e "Launch it from terminal: \033[1;37mrustpdf\033[0m or CLI: \033[1;37mrustpdf-cli --help\033[0m"
