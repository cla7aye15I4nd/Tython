"""
Main test entry point for Tython integration tests.
This file imports and runs various test cases, printing results to stdout.
The integration test framework will run this with both tython and python,
then compare the outputs to ensure compatibility.
"""

from imports.test_runner import run_all_tests

# Run all integration tests
run_all_tests()
