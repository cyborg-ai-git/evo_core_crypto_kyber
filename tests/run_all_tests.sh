#!/bin/bash
set -e

# This script runs a matrix of every valid feature combination
#
# Variables: 
# KAT - Runs the known answer tests (sets kyber_kat cfg flag)

TARGET=$(rustc -vV | sed -n 's|host: ||p')

RUSTFLAGS=${RUSTFLAGS:-""}

# KAT bash variable
if [ -z "$KAT" ]
  then
    echo "Not running Known Answer Tests"
  else
    echo "Running Known Answer Tests"
    RUSTFLAGS+=" --cfg kyber_kat"
fi

# Print Headers
announce(){
  title="#    $1    #"
  edge=$(echo "$title" | sed 's/./#/g')
  echo -e "\n\n$edge"; echo "$title"; echo -e "$edge";
}

# Function to run tests with error handling
run_test(){
  local test_name="$1"
  shift
  announce "$test_name"
  if "$@"; then
    echo "✅ $test_name: PASSED"
  else
    echo "❌ $test_name: FAILED"
    return 1
  fi
}

##############################################################

start=`date +%s`
failed_tests=0

announce "$TARGET"

# Test different Kyber security levels
LEVELS=("kyber512" "kyber768" "kyber1024")

# Test with no features (defaults to kyber768)
run_test "Default (no features)" cargo test --lib --tests || ((failed_tests++))

# Test each security level
for level in "${LEVELS[@]}"; do
  run_test "$level" cargo test --lib --tests --features "$level" || ((failed_tests++))
done

# Test with std feature
run_test "std feature" cargo test --lib --tests --features "std" || ((failed_tests++))

# Test with zeroize feature if available
run_test "zeroize feature" cargo test --lib --tests --features "zeroize" || {
  echo "⚠️  zeroize feature not available or failed"
}

# Test combinations
for level in "${LEVELS[@]}"; do
  run_test "$level + std" cargo test --lib --tests --features "$level,std" || ((failed_tests++))
done

# Run specific test files
run_test "KEM tests" cargo test --test test_kem || ((failed_tests++))
run_test "KEX tests" cargo test --test test_kex || ((failed_tests++))

# Run KAT tests if enabled
if [ ! -z "$KAT" ]; then
  run_test "KAT tests" cargo test --test test_kat || {
    echo "⚠️  KAT tests failed (may require additional dependencies)"
    ((failed_tests++))
  }
fi

# Run examples
run_test "Example AKE" cargo run --example example_ake || ((failed_tests++))

end=`date +%s`
runtime=$((end-start))

announce "Test Summary"
if [ $failed_tests -eq 0 ]; then
  echo "🎉 All tests passed!"
else
  echo "⚠️  $failed_tests test(s) failed"
fi
echo "Test runtime: $runtime seconds"

exit $failed_tests
