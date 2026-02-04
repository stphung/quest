#!/bin/sh
# One-line installer for Quest
# Usage: curl -sSf https://raw.githubusercontent.com/stphung/quest/main/install.sh | sh

set -eu

REPO="stphung/quest"
BINARY="quest"
INSTALL_DIR="${QUEST_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    detect_platform
    check_dependencies
    fetch_latest_release
    download_and_install
    print_success
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
                *) error "Unsupported architecture: $ARCH. Only x86_64 is supported on Linux." ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                x86_64) TARGET="x86_64-apple-darwin" ;;
                arm64)  TARGET="aarch64-apple-darwin" ;;
                *) error "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        *)
            error "Unsupported OS: $OS. Only macOS and Linux are supported."
            ;;
    esac

    echo "Detected platform: $TARGET"
}

check_dependencies() {
    if command -v curl > /dev/null 2>&1; then
        DOWNLOAD="curl -fsSL"
        DOWNLOAD_OUT="curl -fsSL -o"
    elif command -v wget > /dev/null 2>&1; then
        DOWNLOAD="wget -qO-"
        DOWNLOAD_OUT="wget -qO"
    else
        error "curl or wget is required to download Quest."
    fi
}

fetch_latest_release() {
    echo "Fetching latest release..."
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
    ASSET_URL=$(
        $DOWNLOAD "$RELEASE_URL" \
        | grep -o "\"browser_download_url\": *\"[^\"]*${TARGET}\.tar\.gz\"" \
        | head -1 \
        | grep -o 'https://[^"]*'
    ) || error "Could not find a release for $TARGET. Check https://github.com/$REPO/releases"

    echo "Found: $ASSET_URL"
}

download_and_install() {
    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    echo "Downloading..."
    $DOWNLOAD_OUT "$TMPDIR/quest.tar.gz" "$ASSET_URL"

    echo "Extracting..."
    tar xzf "$TMPDIR/quest.tar.gz" -C "$TMPDIR"

    echo "Installing to $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
    mv "$TMPDIR/$BINARY" "$INSTALL_DIR/$BINARY"
    chmod +x "$INSTALL_DIR/$BINARY"
}

print_success() {
    echo ""
    echo "Quest has been installed to $INSTALL_DIR/$BINARY"

    if echo ":$PATH:" | grep -q ":$INSTALL_DIR:"; then
        echo "Run 'quest' to start playing!"
    else
        echo ""
        echo "Add $INSTALL_DIR to your PATH to run 'quest' from anywhere:"
        echo ""
        case "$(basename "${SHELL:-/bin/sh}")" in
            zsh)  echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc && source ~/.zshrc" ;;
            fish) echo "  fish_add_path $INSTALL_DIR" ;;
            *)    echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc && source ~/.bashrc" ;;
        esac
        echo ""
    fi
}

error() {
    echo "Error: $1" >&2
    exit 1
}

main
