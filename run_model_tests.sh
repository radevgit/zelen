#!/bin/bash

# Systematic test runner for all .mzn and .dzn files with timeout protection
# Tests examples, test models, and edge cases

ZELEN_BIN="./target/release/zelen"
EXAMPLES_DIR="examples/models"
TEST_DIR="tests_all/models"
TIMEOUT=10  # seconds per test

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║          Zelen Systematic Model Testing Suite v0.5.0              ║"
echo "║                  With Timeout Protection (${TIMEOUT}s)                  ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
GRAY='\033[0;37m'
NC='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0
TIMEOUT_COUNT=0
SKIPPED_COUNT=0

# Function to test a model
test_model() {
    local mzn_file="$1"
    local dzn_file="$2"
    local model_name=$(basename "$mzn_file" .mzn)
    
    if [ -z "$dzn_file" ] || [ ! -f "$dzn_file" ]; then
        # No valid data file
        printf "  %-50s " "$model_name"
    else
        # With data file
        local data_name=$(basename "$dzn_file" .dzn)
        printf "  %-50s " "${model_name}+${data_name}"
    fi
    
    # Run the model with timeout
    if [ -z "$dzn_file" ] || [ ! -f "$dzn_file" ]; then
        output=$(timeout $TIMEOUT $ZELEN_BIN "$mzn_file" 2>&1)
    else
        output=$(timeout $TIMEOUT $ZELEN_BIN "$mzn_file" "$dzn_file" 2>&1)
    fi
    
    exit_code=$?
    
    # Check timeout (exit code 124)
    if [ $exit_code -eq 124 ]; then
        printf "${YELLOW}⏱ TIMEOUT${NC}\n"
        ((TIMEOUT_COUNT++))
        return
    fi
    
    # Check if it's an error model
    if [[ "$mzn_file" == *"error_"* ]]; then
        if [ $exit_code -ne 0 ]; then
            printf "${GREEN}✓ ERROR${NC} (as expected)\n"
            ((PASS_COUNT++))
            # Optional: show error type
            if [[ "$output" == *"DuplicateDeclaration"* ]]; then
                echo "       └─ DuplicateDeclaration ✓"
            elif [[ "$output" == *"UndefinedVariable"* ]]; then
                echo "       └─ UndefinedVariable ✓"
            elif [[ "$output" == *"ArraySizeMismatch"* ]]; then
                echo "       └─ ArraySizeMismatch ✓"
            elif [[ "$output" == *"TypeError"* ]]; then
                echo "       └─ TypeError ✓"
            elif [[ "$output" == *"InvalidNumber"* ]]; then
                echo "       └─ InvalidNumber ✓"
            fi
        else
            printf "${RED}✗ FAIL${NC} (should error)\n"
            echo "       └─ Unexpectedly succeeded"
            ((FAIL_COUNT++))
        fi
    else
        # Should succeed
        if [ $exit_code -eq 0 ]; then
            printf "${GREEN}✓ PASS${NC}\n"
            ((PASS_COUNT++))
            # Print first line of output
            if [ -n "$output" ]; then
                first_line=$(echo "$output" | head -1)
                echo "       └─ $first_line"
            fi
        else
            printf "${RED}✗ FAIL${NC}\n"
            # Print error message
            if [ -n "$output" ]; then
                # Truncate long output
                short_error=$(echo "$output" | head -1 | cut -c1-70)
                echo "       └─ $short_error"
            fi
            ((FAIL_COUNT++))
        fi
    fi
}

# === SECTION 1: Example Models ===
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 1: Example Models${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

test_model "$EXAMPLES_DIR/test_cli.mzn"
test_model "$EXAMPLES_DIR/test_model.mzn" "$EXAMPLES_DIR/test_data.dzn"
test_model "$EXAMPLES_DIR/test_model2.mzn" "$EXAMPLES_DIR/test_data2.dzn"
test_model "$EXAMPLES_DIR/sudoku.mzn"

echo

# === SECTION 2: Edge Cases ===
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 2: Edge Cases${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/edge_*.mzn; do
    test_model "$mzn"
done

echo

# === SECTION 3: Enum Tests ===
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 3: Enum Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/test_enum_*.mzn; do
    test_model "$mzn"
done

echo

# === SECTION 4: Array Tests ===
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 4: Array Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/test_array*.mzn $TEST_DIR/test_*d_*.mzn $TEST_DIR/test_2d_*.mzn $TEST_DIR/test_3d_*.mzn; do
    if [ -f "$mzn" ]; then
        test_model "$mzn"
    fi
done

echo

# === SECTION 5: Error/Validation Models ===
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 5: Error/Validation Models${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/error_*.mzn; do
    test_model "$mzn"
done

echo

# === SUMMARY ===
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SUMMARY${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

TOTAL=$((PASS_COUNT + FAIL_COUNT + TIMEOUT_COUNT))

printf "  ${GREEN}✓ Passed:${NC}   %3d\n" "$PASS_COUNT"
printf "  ${RED}✗ Failed:${NC}   %3d\n" "$FAIL_COUNT"
printf "  ${YELLOW}⏱ Timeout:${NC}  %3d\n" "$TIMEOUT_COUNT"
printf "  ${GRAY}━ Total:${NC}    %3d\n" "$TOTAL"
echo

if [ $FAIL_COUNT -eq 0 ] && [ $TIMEOUT_COUNT -eq 0 ]; then
    echo -e "  ${GREEN}🎉 All tests passed!${NC}"
    exit 0
elif [ $FAIL_COUNT -eq 0 ]; then
    echo -e "  ${YELLOW}⚠️  All tests passed, but some timed out${NC}"
    exit 1
else
    echo -e "  ${RED}⚠️  Some tests failed!${NC}"
    exit 1
fi
