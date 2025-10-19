#!/bin/bash
# Systematic testing of .mzn/.dzn file integration

cd /home/ross/devpublic/zelen

echo "====== Zelen .mzn/.dzn Integration Tests ======"
echo ""

PASSED=0
FAILED=0
TOTAL=0

# Test helper function
run_test() {
    local test_name="$1"
    local model="$2"
    local data="$3"
    local expected_pattern="$4"
    
    TOTAL=$((TOTAL + 1))
    echo -n "Test $TOTAL: $test_name ... "
    
    if [ -z "$data" ]; then
        result=$(./target/debug/zelen "$model" 2>&1)
    else
        result=$(./target/debug/zelen "$model" "$data" 2>&1)
    fi
    
    if [ $? -ne 0 ]; then
        echo "FAILED (exit code)"
        echo "  Error: $result"
        FAILED=$((FAILED + 1))
        return 1
    fi
    
    if [ -z "$expected_pattern" ] || echo "$result" | grep -q "$expected_pattern"; then
        echo "PASSED"
        if [ ! -z "$expected_pattern" ]; then
            echo "  Found: $expected_pattern"
        fi
        PASSED=$((PASSED + 1))
        return 0
    else
        echo "FAILED (pattern mismatch)"
        echo "  Expected pattern: $expected_pattern"
        echo "  Got: $result"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# Test 1: Simple model with data file
run_test "Simple array with constraints" \
    "examples/models/test_model.mzn" \
    "examples/models/test_data.dzn" \
    "choice = [1-5]"

# Test 2: Create additional test files
cat > /tmp/test_knapsack.mzn << 'EOF'
% Knapsack problem with data file
int: n;
int: capacity;
array[1..n] of int: weights;
array[1..n] of int: values;

var 1..0: total_weight;
var 1..0: total_value;
array[1..n] of var 0..1: items;

constraint total_weight = sum(i in 1..n)(items[i] * weights[i]);
constraint total_value = sum(i in 1..n)(items[i] * values[i]);
constraint total_weight <= capacity;
solve maximize total_value;
EOF

cat > /tmp/test_knapsack.dzn << 'EOF'
n = 3;
capacity = 10;
weights = [4, 5, 6];
values = [10, 15, 20];
EOF

# Test 3: Model without data file (inline parameters)
cat > /tmp/test_inline_params.mzn << 'EOF'
% Model with inline parameters (no data file needed)
int: n = 4;
array[1..n] of int: costs = [10, 20, 30, 40];

var 1..n: choice;
constraint costs[choice] <= 25;
solve satisfy;
EOF

# Test 4: Multi-dimensional array parameter
cat > /tmp/test_2d_param.mzn << 'EOF'
% 2D array with data file
int: rows;
int: cols;
array[1..rows, 1..cols] of int: matrix;

var 1..rows: r;
var 1..cols: c;
constraint matrix[r, c] > 5;
solve satisfy;
EOF

cat > /tmp/test_2d_param.dzn << 'EOF'
rows = 3;
cols = 3;
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
EOF

run_test "Model without data file" \
    "/tmp/test_inline_params.mzn" \
    "" \
    "choice ="

run_test "2D array parameters" \
    "/tmp/test_2d_param.mzn" \
    "/tmp/test_2d_param.dzn" \
    "[rc] = [1-3]"

# Test 5: Parameter array with different types
cat > /tmp/test_float_param.mzn << 'EOF'
% Float parameters with data file
int: n;
array[1..n] of float: costs_f;

var 1..n: choice;
constraint costs_f[choice] <= 15.5;
solve satisfy;
EOF

cat > /tmp/test_float_param.dzn << 'EOF'
n = 3;
costs_f = [10.5, 20.3, 14.2];
EOF

run_test "Float array parameters" \
    "/tmp/test_float_param.mzn" \
    "/tmp/test_float_param.dzn" \
    "choice ="

# Test 6: Bool parameters
cat > /tmp/test_bool_param.mzn << 'EOF'
% Bool parameters with data file
int: n;
array[1..n] of bool: flags;

var 1..n: idx;
constraint flags[idx] = true;
solve satisfy;
EOF

cat > /tmp/test_bool_param.dzn << 'EOF'
n = 3;
flags = [false, true, false];
EOF

run_test "Boolean array parameters" \
    "/tmp/test_bool_param.mzn" \
    "/tmp/test_bool_param.dzn" \
    "idx = 2"

# Test 7: Multiple parameter types in one model
cat > /tmp/test_mixed_params.mzn << 'EOF'
% Mixed parameter types
int: n;
array[1..n] of int: int_array;
array[1..n] of float: float_array;
array[1..n] of bool: bool_array;

var 1..n: x;
constraint int_array[x] > 5;
constraint float_array[x] < 20.0;
constraint bool_array[x] = true;
solve satisfy;
EOF

cat > /tmp/test_mixed_params.dzn << 'EOF'
n = 3;
int_array = [4, 10, 6];
float_array = [15.0, 18.5, 19.0];
bool_array = [false, true, true];
EOF

run_test "Mixed parameter types" \
    "/tmp/test_mixed_params.mzn" \
    "/tmp/test_mixed_params.dzn" \
    "x = [1-3]"

echo ""
echo "====== Test Summary ======"
echo "Total: $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $FAILED -eq 0 ]; then
    echo "Result: ALL TESTS PASSED ✓"
    exit 0
else
    echo "Result: SOME TESTS FAILED ✗"
    exit 1
fi
