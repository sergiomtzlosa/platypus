#!/bin/bash

# Build script for Platypus compiler
# Handles compilation in debug and release modes

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
BUILD_MODE="debug"
VERBOSE=false

# Functions
print_usage() {
    cat << EOF
Usage: ./build.sh [OPTION]

Build the Platypus compiler from source.

OPTIONS:
    release         Build in release mode (optimized, slower compile)
    debug           Build in debug mode (faster compile, no optimizations)
    clean           Remove build artifacts
    install         Build and install to /usr/local/bin
    test            Run all examples
    help            Show this help message

EXAMPLES:
    ./build.sh              # Build in debug mode
    ./build.sh release      # Build optimized release binary
    ./build.sh clean        # Clean build artifacts
    ./build.sh install      # Build and install globally
    ./build.sh test         # Test all examples

EOF
}

build_debug() {
    echo -e "${BLUE}Building Platypus in debug mode...${NC}"
    cargo build
    echo -e "${GREEN}✓ Build complete!${NC}"
    echo -e "Binary location: ${BLUE}$SCRIPT_DIR/target/debug/platypus${NC}"
}

build_release() {
    echo -e "${BLUE}Building Platypus in release mode...${NC}"
    echo -e "${YELLOW}This may take a few minutes...${NC}"
    cargo build --release
    echo -e "${GREEN}✓ Build complete!${NC}"
    echo -e "Binary location: ${BLUE}$SCRIPT_DIR/target/release/platypus${NC}"
}

clean() {
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}✓ Clean complete!${NC}"
}

install_binary() {
    echo -e "${BLUE}Building in release mode for installation...${NC}"
    cargo build --release
    
    if [ ! -f "$SCRIPT_DIR/target/release/platypus" ]; then
        echo -e "${RED}Error: Build failed${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}Installing to /usr/local/bin...${NC}"
    sudo cp "$SCRIPT_DIR/target/release/platypus" /usr/local/bin/platypus
    sudo chmod +x /usr/local/bin/platypus
    
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo -e "You can now run ${BLUE}platypus${NC} from anywhere"
    echo ""
    echo "Verify installation:"
    echo -e "  ${BLUE}platypus --version${NC}"
    echo -e "  ${BLUE}platypus --help${NC}"
}

run_tests() {
    if [ ! -f "$SCRIPT_DIR/target/release/platypus" ]; then
        echo -e "${YELLOW}Release binary not found. Building first...${NC}"
        build_release
    fi
    
    if [ -f "$SCRIPT_DIR/test_all_examples.sh" ]; then
        echo -e "${BLUE}Running all examples...${NC}"
        "$SCRIPT_DIR/test_all_examples.sh"
    else
        echo -e "${RED}Error: test_all_examples.sh not found${NC}"
        exit 1
    fi
}

# Main script
cd "$SCRIPT_DIR"

case "${1:-debug}" in
    release)
        build_release
        ;;
    debug)
        build_debug
        ;;
    clean)
        clean
        ;;
    install)
        install_binary
        ;;
    test)
        run_tests
        ;;
    help|--help|-h)
        print_usage
        ;;
    *)
        echo -e "${RED}Error: Unknown option '$1'${NC}"
        echo ""
        print_usage
        exit 1
        ;;
esac
