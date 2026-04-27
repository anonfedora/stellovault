#!/bin/bash

# StelloVault Comprehensive Testing Script
# Runs all test suites and generates reports

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     StelloVault Smart Contract Testing Infrastructure         ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run test suite
run_test_suite() {
    local suite_name=$1
    local test_pattern=$2
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Running: ${suite_name}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    if cargo test ${test_pattern} -- --nocapture --test-threads=1 2>&1 | tee test_output.log; then
        echo -e "${GREEN}✓ ${suite_name} PASSED${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ ${suite_name} FAILED${NC}"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
    echo ""
}

# 1. Unit Tests
run_test_suite "Unit Tests" "unit_"

# 2. Integration Tests
run_test_suite "Integration Tests" "integration_"

# 3. Property-Based Tests
run_test_suite "Property-Based Tests" "property_"

# 4. Security Tests
run_test_suite "Security Tests" "security_"

# 5. Attack Vector Tests
run_test_suite "Attack Vector Tests" "attack_"

# 6. Gas Optimization Tests
run_test_suite "Gas Optimization Tests" "gas_"

# 7. Formal Verification Tests
run_test_suite "Formal Verification Tests" "invariant_"

# 8. Economic Simulation Tests
run_test_suite "Economic Simulation Tests" "economic_"

# 9. Performance Tests
run_test_suite "Performance Tests" "performance_"

# 10. All Tests
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Running: All Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if cargo test -- --nocapture 2>&1 | tee all_tests.log; then
    echo -e "${GREEN}✓ All Tests PASSED${NC}"
    ((PASSED_TESTS++))
else
    echo -e "${RED}✗ All Tests FAILED${NC}"
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))
echo ""

# Security Audit
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Running: Security Audit${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if command -v cargo-audit &> /dev/null; then
    if cargo audit 2>&1 | tee audit.log; then
        echo -e "${GREEN}✓ Security Audit PASSED${NC}"
    else
        echo -e "${YELLOW}⚠ Security Audit found issues (see audit.log)${NC}"
    fi
else
    echo -e "${YELLOW}⚠ cargo-audit not installed, skipping${NC}"
fi
echo ""

# Code Coverage
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Generating: Code Coverage Report${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if command -v cargo-tarpaulin &> /dev/null; then
    if cargo tarpaulin --out Html --timeout 300 2>&1 | tee coverage.log; then
        echo -e "${GREEN}✓ Coverage Report Generated${NC}"
        echo -e "${YELLOW}  Report: tarpaulin-report.html${NC}"
    else
        echo -e "${YELLOW}⚠ Coverage report generation had issues${NC}"
    fi
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin not installed, skipping${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                      TEST SUMMARY                             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Total Test Suites:  ${TOTAL_TESTS}"
echo -e "Passed:             ${GREEN}${PASSED_TESTS}${NC}"
echo -e "Failed:             ${RED}${FAILED_TESTS}${NC}"
echo ""

# Final status
if [ ${FAILED_TESTS} -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    exit 1
fi
