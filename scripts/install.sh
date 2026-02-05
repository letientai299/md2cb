#!/usr/bin/env bash
set -euo pipefail

REPO="letientai299/md2cb"
BINARY_NAME="md2cb"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $*" >&2; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Download and install md2cb from GitHub.

OPTIONS:
    -m, --from-main    Download latest build from main branch (requires gh CLI)
    -d, --dir DIR      Installation directory (default: /usr/local/bin)
    -h, --help         Show this help message

EXAMPLES:
    $(basename "$0")              # Install latest release
    $(basename "$0") --from-main  # Install latest build from main branch
    $(basename "$0") -d ~/.local/bin  # Install to custom directory
EOF
}

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        *)       error "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x64" ;;
        arm64|aarch64)
            if [[ "$os" == "macos" ]]; then
                arch="arm64"
            else
                error "ARM64 Linux is not currently supported"
            fi
            ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

download_latest_release() {
    local platform="$1"
    local artifact_name="${BINARY_NAME}-${platform}.tar.gz"

    info "Fetching latest release..."
    local release_url
    release_url=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep "browser_download_url.*${artifact_name}" | \
        cut -d '"' -f 4)

    if [[ -z "$release_url" ]]; then
        error "Could not find release artifact for platform: ${platform}"
    fi

    info "Downloading ${artifact_name}..."
    curl -sL "$release_url" -o "/tmp/${artifact_name}"
    echo "/tmp/${artifact_name}"
}

download_from_main() {
    local platform="$1"
    local artifact_name="${BINARY_NAME}-${platform}"

    if ! command -v gh &>/dev/null; then
        error "GitHub CLI (gh) is required for --from-main. Install from: https://cli.github.com/"
    fi

    if ! gh auth status &>/dev/null; then
        error "GitHub CLI not authenticated. Run: gh auth login"
    fi

    info "Fetching latest successful build from main branch..."

    # Get the latest successful workflow run on main
    local run_id
    run_id=$(gh run list --repo "$REPO" --branch main --workflow ci.yml --status success --limit 1 --json databaseId --jq '.[0].databaseId')

    if [[ -z "$run_id" || "$run_id" == "null" ]]; then
        error "No successful workflow runs found on main branch"
    fi

    info "Found workflow run: ${run_id}"
    info "Downloading artifact: ${artifact_name}..."

    local tmp_dir="/tmp/md2cb-download-$$"
    mkdir -p "$tmp_dir"

    gh run download "$run_id" --repo "$REPO" --name "$artifact_name" --dir "$tmp_dir"

    # Find the downloaded archive
    local archive
    archive=$(find "$tmp_dir" -name "*.tar.gz" -type f | head -1)

    if [[ -z "$archive" ]]; then
        error "Could not find downloaded archive in ${tmp_dir}"
    fi

    echo "$archive"
}

install_binary() {
    local archive="$1"
    local install_dir="$2"

    info "Extracting archive..."
    local tmp_extract="/tmp/md2cb-extract-$$"
    mkdir -p "$tmp_extract"
    tar -xzf "$archive" -C "$tmp_extract"

    # Find the binary
    local binary
    binary=$(find "$tmp_extract" -name "$BINARY_NAME" -type f | head -1)

    if [[ -z "$binary" ]]; then
        error "Could not find ${BINARY_NAME} binary in archive"
    fi

    # Create install directory if needed
    if [[ ! -d "$install_dir" ]]; then
        info "Creating directory: ${install_dir}"
        mkdir -p "$install_dir"
    fi

    # Check if we need sudo
    if [[ -w "$install_dir" ]]; then
        info "Installing to ${install_dir}/${BINARY_NAME}..."
        cp "$binary" "${install_dir}/${BINARY_NAME}"
        chmod +x "${install_dir}/${BINARY_NAME}"
    else
        info "Installing to ${install_dir}/${BINARY_NAME} (requires sudo)..."
        sudo cp "$binary" "${install_dir}/${BINARY_NAME}"
        sudo chmod +x "${install_dir}/${BINARY_NAME}"
    fi

    # Cleanup
    rm -rf "$tmp_extract" "$archive"

    info "Successfully installed ${BINARY_NAME} to ${install_dir}/${BINARY_NAME}"

    # Verify installation
    if command -v "$BINARY_NAME" &>/dev/null; then
        info "Version: $(${BINARY_NAME} --version 2>/dev/null || echo 'unknown')"
    else
        warn "${install_dir} may not be in your PATH"
    fi
}

main() {
    local from_main=false
    local install_dir="$INSTALL_DIR"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -m|--from-main)
                from_main=true
                shift
                ;;
            -d|--dir)
                install_dir="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1. Use --help for usage."
                ;;
        esac
    done

    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"

    local archive
    if [[ "$from_main" == true ]]; then
        archive=$(download_from_main "$platform")
    else
        archive=$(download_latest_release "$platform")
    fi

    install_binary "$archive" "$install_dir"
}

main "$@"
