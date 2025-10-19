#!/bin/bash

# Comprehensive test runner for all .mzn and .dzn files
# Systematically tests examples, test models, and edge cases

set -e

ZELEN_BIN="./target/release/zelen"
EXAMPLES_DIR="examples/models"
TEST_DIR="tests_all/models"

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║          Zelen Systematic Model Testing Suite v0.5.0              ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PASS_COUNT=0
FAIL_COUNT=0
ERROR_COUNT=0

# Function to test a model
test_model() {
    local mzn_file="$1"
    local dzn_file="$2"
    local model_name=$(basename "$mzn_file" .mzn)
    
    if [ -z "$dzn_file" ]; then
        # No data file
        printf "  %-50s " "$model_name"
    else
        # With data file
        local data_name=$(basename "$dzn_file" .dzn)
        printf "  %-50s " "${model_name}+${data_name}"
    fi
    
    # Run the model
    if [ -z "$dzn_file" ]; then
        output=$($ZELEN_BIN "$mzn_file" 2>&1)
    else
        output=$($ZELEN_BIN "$mzn_file" "$dzn_file" 2>&1)
    fi
    
    exit_code=$?
    
    # Check if it's an error model
    if [[ "$mzn_file" == *"error_"* ]]; then
        if [ $exit_code -ne 0 ]; then
            printf "${GREEN}✓ ERROR${NC} (as expected)\n"
            ((PASS_COUNT++))
        else
            printf "${RED}✗ FAIL${NC} (should error)\n"
            echo "       Output: $output"
            ((FAIL_COUNT++))
        fi
    else
        # Should succeed
        if [ $exit_code -eq 0 ]; then
            printf "${GREEN}✓ PASS${NC}\n"
            ((PASS_COUNT++))
            # Optionally print first line of output
            if [ -n "$output" ]; then
                first_line=$(echo "$output" | head -1)
                echo "       → $first_line"
            fi
        else
            printf "${RED}✗ FAIL${NC}\n"
            echo "       Error: $output"
            ((FAIL_COUNT++))
        fi
    fi
}

# Test Example Models
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 1: Example Models${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# test_model.mzn + test_data.dzn
test_model "$EXAMPLES_DIR/test_model.mzn" "$EXAMPLES_DIR/test_data.dzn"

# test_model2.mzn + test_data2.dzn
test_model "$EXAMPLES_DIR/test_model2.mzn" "$EXAMPLES_DIR/test_data2.dzn"

# test_cli.mzn (no data file)
test_model "$EXAMPLES_DIR/test_cli.mzn"

# sudoku.mzn (no data file)
test_model "$EXAMPLES_DIR/sudoku.mzn"

echo

# Test Edge Cases
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 2: Edge Cases${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/edge_*.mzn; do
    test_model "$mzn"
done

echo

# Test Enum Models
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 3: Enum Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/test_enum_*.mzn; do
    test_model "$mzn"
done

echo

# Test Array Models
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 4: Array Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/test_array*.mzn $TEST_DIR/test_*d_*.mzn; do
    if [ -f "$mzn" ]; then
        test_model "$mzn"
    fi
done

echo

# Test Error Models
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SECTION 5: Error/Validation Models${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

for mzn in $TEST_DIR/error_*.mzn; do
    test_model "$mzn"
done

echo

# Print summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SUMMARY${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

TOTAL=$((PASS_COUNT + FAIL_COUNT))

printf "  ${GREEN}✓ Passed:${NC}  %3d\n" "$PASS_COUNT"
printf "  ${RED}✗ Failed:${NC}  %3d\n" "$FAIL_COUNT"
printf "  ${YELLOW}━ Total:${NC}   %3d\n" "$TOTAL"
echo

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "  ${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "  ${RED}⚠️  Some tests failed!${NC}"
    exit 1
fi
