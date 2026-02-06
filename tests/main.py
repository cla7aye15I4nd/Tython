"""
Main test entry point for Tython integration tests.
This file imports and runs various test cases, printing results to stdout.
The integration test framework will run this with both tython and python,
then compare the outputs to ensure compatibility.
"""

import test_simple
import test_imports

def run_all_tests():
    print("=== Running Tython Integration Tests ===")

    # Test 1: Simple function without imports
    print("\n[Test 1: Simple Function]")
    result1 = test_simple.factorial(5)
    print(f"factorial(5) = {result1}")
    assert result1 == 120, f"Expected 120, got {result1}"
    print("PASS")

    # Test 2: Module imports and dependencies
    print("\n[Test 2: Module Imports]")
    result2 = test_imports.compute(10)
    print(f"compute(10) = {result2}")
    assert result2 == 21, f"Expected 21, got {result2}"
    print("PASS")

    print("\n=== All Tests Passed ===")

if __name__ == "__main__":
    run_all_tests()
