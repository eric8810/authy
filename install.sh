#!/bin/sh
# Authy install script — Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/eric8810/authy/main/install.sh | sh
set -eu

REPO="eric8810/authy"
BINARY="authy"

main() {
    detect_platform
    get_version
    download_and_install
    verify_install
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS_TARGET="unknown-linux-gnu" ;;
        Darwin) OS_TARGET="apple-darwin" ;;
        *)      error "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_TARGET="x86_64" ;;
        aarch64|arm64)   ARCH_TARGET="aarch64" ;;
        *)               error "Unsupported architecture: $ARCH" ;;
    esac

    TARGET="${ARCH_TARGET}-${OS_TARGET}"
    info "Detected platform: $TARGET"
}

get_version() {
    if [ -n "${AUTHY_VERSION:-}" ]; then
        VERSION="$AUTHY_VERSION"
        info "Using specified version: $VERSION"
        return
    fi

    info "Fetching latest version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        error "Failed to determine latest version"
    fi

    info "Latest version: $VERSION"
}

download_and_install() {
    ARCHIVE="${BINARY}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    info "Downloading $URL..."
    curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"

    info "Extracting..."
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

    # Determine install directory
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi

    install -m 755 "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    info "Installed to ${INSTALL_DIR}/${BINARY}"

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH."
            warn "Add it with: export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
    esac
}

verify_install() {
    if command -v "$BINARY" >/dev/null 2>&1; then
        info "Verification: $("$BINARY" --version 2>/dev/null || echo "$BINARY installed")"
    else
        info "Install complete. Restart your shell or update PATH to use '$BINARY'."
    fi
}

info() {
    printf '\033[1;32m%s\033[0m\n' "$*" >&2
}

warn() {
    printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2
}

error() {
    printf '\033[1;31merror:\033[0m %s\n' "$*" >&2
    exit 1
}

main
