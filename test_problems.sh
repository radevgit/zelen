#!/bin/bash
# Test script for Zelen solver with hakank problems
# Usage: ./test_problems.sh [number_of_problems]

cd "$(dirname "$0")/zinc/hakank" || exit 1

TIMEOUT=30
TIME_LIMIT=25000
NUM_PROBLEMS=${1:-50}

echo "=== Testing $NUM_PROBLEMS problems with Zelen solver ==="
echo "Timeout: ${TIMEOUT}s, Time limit: ${TIME_LIMIT}ms"
echo ""

passed=0
failed=0
skipped=0

# Get list of .mzn files
mzn_files=$(ls *.mzn | head -$NUM_PROBLEMS)

for mzn in $mzn_files; do
    base="${mzn%.mzn}"
    
    # Check if there are numbered data files (e.g., problem1.dzn, problem2.dzn)
    dzn_files=$(ls ${base}[0-9]*.dzn 2>/dev/null | sort)
    
    if [ -n "$dzn_files" ]; then
        # Multiple data files - test with first one only for speed
        dzn=$(echo "$dzn_files" | head -1)
        printf "%-45s [%s] " "$mzn" "$(basename $dzn)"
        result=$(timeout $TIMEOUT minizinc --solver zelen --time-limit $TIME_LIMIT "$mzn" "$dzn" 2>&1)
    elif [ -f "${base}.dzn" ]; then
        # Single data file with same name
        printf "%-45s [.dzn] " "$mzn"
        result=$(timeout $TIMEOUT minizinc --solver zelen --time-limit $TIME_LIMIT "$mzn" "${base}.dzn" 2>&1)
    else
        # No data file needed
        printf "%-45s        " "$mzn"
        result=$(timeout $TIMEOUT minizinc --solver zelen --time-limit $TIME_LIMIT "$mzn" 2>&1)
    fi
    
    exit_code=$?
    
    if [ $exit_code -eq 124 ]; then
        echo "⏱ TIMEOUT"
        ((failed++))
    elif echo "$result" | grep -q "=========="; then
        echo "✓ SOLVED"
        ((passed++))
    elif echo "$result" | grep -q "=====UNSATISFIABLE====="; then
        echo "✓ UNSAT"
        ((passed++))
    elif echo "$result" | grep -qi "error"; then
        echo "✗ ERROR"
        ((failed++))
        # Uncomment to see error details:
        # echo "$result" | grep -i error | head -3
    else
        echo "✗ FAILED"
        ((failed++))
    fi
done

echo ""
echo "========================================="
echo "RESULTS: $passed passed, $failed failed"
echo "Success rate: $(( passed * 100 / NUM_PROBLEMS ))%"
echo "========================================="
