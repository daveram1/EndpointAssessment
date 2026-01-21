#!/bin/bash
set -e

# Cross-platform build script for Endpoint Assessment Agent
# Builds installer packages for Linux (DEB, RPM), macOS (PKG), and Windows (MSI)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="${VERSION:-0.1.0}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    echo "Usage: $0 [OPTIONS] [TARGETS...]"
    echo ""
    echo "Targets:"
    echo "  linux-deb    Build Debian package (.deb)"
    echo "  linux-rpm    Build RPM package (.rpm)"
    echo "  macos-pkg    Build macOS package (.pkg)"
    echo "  windows-msi  Build Windows installer (.msi) - requires cross-compilation"
    echo "  all          Build all packages for current platform"
    echo ""
    echo "Options:"
    echo "  -v, --version VERSION  Set package version (default: $VERSION)"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 linux-deb                    # Build DEB package"
    echo "  $0 -v 1.0.0 linux-deb linux-rpm # Build DEB and RPM with version 1.0.0"
    echo "  $0 all                          # Build all packages for current OS"
}

check_rust() {
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Install from https://rustup.rs"
        exit 1
    fi
    log_info "Rust version: $(rustc --version)"
}

build_agent() {
    local target="$1"
    log_info "Building agent binary${target:+ for $target}..."

    cd "$PROJECT_ROOT"

    if [ -n "$target" ]; then
        cargo build --release -p agent --target "$target"
    else
        cargo build --release -p agent
    fi

    log_success "Agent binary built successfully"
}

build_linux_deb() {
    log_info "Building Debian package..."

    if ! command -v cargo-deb &> /dev/null; then
        log_warn "cargo-deb not found. Installing..."
        cargo install cargo-deb
    fi

    cd "$PROJECT_ROOT"

    # Copy cargo-deb config to agent's Cargo.toml temporarily
    if ! grep -q '\[package.metadata.deb\]' agent/Cargo.toml; then
        cat packaging/linux/deb/cargo-deb.toml >> agent/Cargo.toml
    fi

    cargo deb -p agent --no-build

    local deb_file=$(ls -t target/debian/*.deb 2>/dev/null | head -1)
    if [ -n "$deb_file" ]; then
        log_success "DEB package created: $deb_file"
    else
        log_error "Failed to create DEB package"
        return 1
    fi
}

build_linux_rpm() {
    log_info "Building RPM package..."

    if ! command -v rpmbuild &> /dev/null; then
        log_error "rpmbuild not found. Install rpm-build package."
        log_info "  Ubuntu/Debian: sudo apt install rpm"
        log_info "  Fedora/RHEL: sudo dnf install rpm-build"
        return 1
    fi

    cd "$PROJECT_ROOT"

    # Create tarball for rpmbuild
    local tarball_name="endpoint-agent-$VERSION"
    mkdir -p ~/rpmbuild/{SOURCES,SPECS}

    tar czf ~/rpmbuild/SOURCES/$tarball_name.tar.gz \
        --transform "s,^,$tarball_name/," \
        --exclude='target' \
        --exclude='.git' \
        .

    # Copy spec file
    sed "s/Version:.*/Version:        $VERSION/" \
        packaging/linux/rpm/endpoint-agent.spec > ~/rpmbuild/SPECS/endpoint-agent.spec

    # Build RPM
    rpmbuild -bb ~/rpmbuild/SPECS/endpoint-agent.spec

    local rpm_file=$(ls -t ~/rpmbuild/RPMS/*/*.rpm 2>/dev/null | head -1)
    if [ -n "$rpm_file" ]; then
        cp "$rpm_file" target/release/
        log_success "RPM package created: target/release/$(basename $rpm_file)"
    else
        log_error "Failed to create RPM package"
        return 1
    fi
}

build_macos_pkg() {
    log_info "Building macOS package..."

    if [[ "$(uname)" != "Darwin" ]]; then
        log_error "macOS packages can only be built on macOS"
        return 1
    fi

    cd "$PROJECT_ROOT"
    chmod +x packaging/macos/pkg/build-pkg.sh
    VERSION="$VERSION" packaging/macos/pkg/build-pkg.sh
}

build_windows_msi() {
    log_info "Building Windows MSI package..."

    if [[ "$(uname)" == "MINGW"* ]] || [[ "$(uname)" == "MSYS"* ]] || [[ "$(uname)" == "CYGWIN"* ]]; then
        # Running on Windows
        cd "$PROJECT_ROOT/packaging/windows/wix"
        powershell.exe -ExecutionPolicy Bypass -File build-msi.ps1 -Version "$VERSION"
    else
        log_warn "Windows MSI can only be built on Windows."
        log_info "To build on Linux, you can cross-compile the binary:"
        log_info "  1. Install cross-compilation target:"
        log_info "     rustup target add x86_64-pc-windows-gnu"
        log_info "  2. Build: cargo build --release -p agent --target x86_64-pc-windows-gnu"
        log_info "  3. Copy binary to Windows and run build-msi.ps1"

        # Offer to build cross-compiled binary
        read -p "Build cross-compiled Windows binary? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
                rustup target add x86_64-pc-windows-gnu
            fi
            build_agent "x86_64-pc-windows-gnu"
            log_success "Windows binary created: target/x86_64-pc-windows-gnu/release/agent.exe"
            log_info "Copy this to Windows and run build-msi.ps1 to create the MSI"
        fi
    fi
}

# Parse arguments
TARGETS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        linux-deb|linux-rpm|macos-pkg|windows-msi|all)
            TARGETS+=("$1")
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Default to 'all' if no targets specified
if [ ${#TARGETS[@]} -eq 0 ]; then
    TARGETS=("all")
fi

# Expand 'all' based on current OS
if [[ " ${TARGETS[*]} " =~ " all " ]]; then
    TARGETS=()
    case "$(uname)" in
        Linux)
            TARGETS+=("linux-deb" "linux-rpm")
            ;;
        Darwin)
            TARGETS+=("macos-pkg")
            ;;
        MINGW*|MSYS*|CYGWIN*)
            TARGETS+=("windows-msi")
            ;;
    esac
fi

log_info "Building Endpoint Assessment Agent v$VERSION"
log_info "Targets: ${TARGETS[*]}"
echo ""

# Check prerequisites
check_rust

# Build the agent binary first
build_agent

# Build each target
for target in "${TARGETS[@]}"; do
    echo ""
    case $target in
        linux-deb)
            build_linux_deb
            ;;
        linux-rpm)
            build_linux_rpm
            ;;
        macos-pkg)
            build_macos_pkg
            ;;
        windows-msi)
            build_windows_msi
            ;;
    esac
done

echo ""
log_success "Build complete!"
echo ""
log_info "Packages are in: $PROJECT_ROOT/target/release/"
