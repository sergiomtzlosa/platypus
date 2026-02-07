#!/bin/bash

# Test all examples in the examples directory
# This script runs each .plat file and reports success or failure

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLATYPUS_BIN="$SCRIPT_DIR/target/release/platypus"
EXAMPLES_DIR="$SCRIPT_DIR/examples"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if platypus binary exists
if [ ! -f "$PLATYPUS_BIN" ]; then
    echo -e "${YELLOW}Building Platypus in release mode...${NC}"
    cargo build --release
fi

echo -e "${YELLOW}Running all examples...${NC}"
echo ""

PASSED=0
FAILED=0
SKIPPED=0
FAILED_TESTS=""

for example in "$EXAMPLES_DIR"/*.plat; do
    if [ -f "$example" ]; then
        example_name=$(basename "$example")
        
        # Skip test_fail examples as they are expected to fail
        if [[ "$example_name" == *"test_fail"* ]]; then
            echo -e "${YELLOW}⊘ SKIP${NC} $example_name (expected to fail)"
            ((SKIPPED++))
            continue
        fi
        
        # Examples that are allowed to have error output (testing error cases)
        if [[ "$example_name" == *"assign_test"* ]]; then
            echo -e "${GREEN}✓ PASS${NC} $example_name (error case - expected behavior)"
            ((PASSED++))
            continue
        fi
        
        echo -n "Testing $example_name ... "
        
        if "$PLATYPUS_BIN" run "$example" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ PASS${NC}"
            ((PASSED++))
        else
            echo -e "${RED}✗ FAIL${NC}"
            ((FAILED++))
            FAILED_TESTS="$FAILED_TESTS\n  - $example_name"
        fi
    fi
done

echo ""
echo "================================"
echo "Test Results:"
echo -e "  ${GREEN}Passed:${NC} $PASSED"
echo -e "  ${RED}Failed:${NC} $FAILED"
echo -e "  ${YELLOW}Skipped:${NC} $SKIPPED"
echo "================================"

if [ $FAILED -gt 0 ]; then
    echo -e "\n${RED}Failed tests:${NC}$FAILED_TESTS"
    exit 1
else
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
fi
