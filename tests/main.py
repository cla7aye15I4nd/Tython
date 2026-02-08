"""
Main test entry point for Tython integration tests.
This file imports and runs various test cases, printing results to stdout.
The integration test framework will run this with both tython and python,
then compare the outputs to ensure compatibility.
"""

# Test various import patterns - simple sibling imports
from imports import module_a
from imports import module_b
from imports import module_c
from imports import module_d

# Complex nested imports
from imports import module_e
from imports import module_f
from imports import module_g

# Direct nested module imports
from imports.nested.deep.deeper import bottom
from imports.nested.deep import mid_level
from imports.nested import top_level

def run_all_tests():
    print("=== Running Tython Import Tests ===")

    # Test 1: Simple module import (no dependencies)
    print("\n[Test 1: Module A - No Dependencies]")
    result1 = module_a.func_a()
    print(f"module_a.func_a() = {result1}")
    assert result1 == "A", f"Expected 'A', got {result1}"
    result1b = module_a.compute_a(10)
    print(f"module_a.compute_a(10) = {result1b}")
    assert result1b == 11, f"Expected 11, got {result1b}"
    print("PASS")

    # Test 2: Sibling import (B imports A)
    print("\n[Test 2: Module B - Imports Sibling A]")
    result2 = module_b.func_b()
    print(f"module_b.func_b() = {result2}")
    assert result2 == "BA", f"Expected 'BA', got {result2}"
    result2b = module_b.compute_b(5)
    print(f"module_b.compute_b(5) = {result2b}")
    assert result2b == 12, f"Expected 12, got {result2b}"
    print("PASS")

    # Test 3: Multiple sibling imports (C imports A and B)
    print("\n[Test 3: Module C - Imports Siblings A and B]")
    result3 = module_c.func_c()
    print(f"module_c.func_c() = {result3}")
    assert result3 == "CABA", f"Expected 'CABA', got {result3}"
    result3b = module_c.compute_c(3)
    print(f"module_c.compute_c(3) = {result3b}")
    assert result3b == 12, f"Expected 12, got {result3b}"
    print("PASS")

    # Test 4: Deep dependency chain (D imports C, which imports B, which imports A)
    print("\n[Test 4: Module D - Deep Dependency Chain]")
    result4 = module_d.func_d()
    print(f"module_d.func_d() = {result4}")
    assert result4 == "DCABA", f"Expected 'DCABA', got {result4}"
    result4b = module_d.compute_d(2)
    print(f"module_d.compute_d(2) = {result4b}")
    assert result4b == 12, f"Expected 12, got {result4b}"
    print("PASS")

    # Test 5: Nested package imports (from nested.deep.deeper.bottom import X)
    print("\n[Test 5: Module E - Complex Nested Imports]")
    result5 = module_e.module_e_func()
    print(f"module_e.module_e_func() = {result5}")
    assert result5 == "E_bottom_mid_bottom_top_mid_bottom", f"Expected 'E_bottom_mid_bottom_top_mid_bottom', got {result5}"
    result5b = module_e.module_e_class()
    print(f"module_e.module_e_class() = {result5b}")
    assert result5b == "bottom_class_method", f"Expected 'bottom_class_method', got {result5b}"
    result5c = module_e.module_e_const()
    print(f"module_e.module_e_const() = {result5c}")
    assert result5c == 226, f"Expected 226, got {result5c}"
    print("PASS")

    # Test 6: Full nested module path imports (import nested.deep.deeper.bottom)
    print("\n[Test 6: Module F - Full Nested Module Paths]")
    result6 = module_f.module_f_func()
    print(f"module_f.module_f_func() = {result6}")
    assert result6 == "F_bottom_mid_bottom_top_mid_bottom", f"Expected 'F_bottom_mid_bottom_top_mid_bottom', got {result6}"
    result6b = module_f.module_f_access()
    print(f"module_f.module_f_access() = {result6b}")
    assert result6b == 126, f"Expected 126, got {result6b}"
    print("PASS")

    # Test 7: Mixed import styles with aliases
    print("\n[Test 7: Module G - Mixed Import Styles]")
    result7 = module_g.module_g_func()
    print(f"module_g.module_g_func() = {result7}")
    expected7 = "G_sibling_b_top_mid_bottom_sibling_a_calls_top_mid_bottom_sibling_a_calls_top_mid_bottom"
    assert result7 == expected7, f"Expected '{expected7}', got {result7}"
    result7b = module_g.module_g_use_e()
    print(f"module_g.module_g_use_e() = {result7b}")
    assert result7b == "E_bottom_mid_bottom_top_mid_bottom", f"Expected 'E_bottom_mid_bottom_top_mid_bottom', got {result7b}"
    print("PASS")

    # Test 8: Direct access to deeply nested modules
    print("\n[Test 8: Direct Nested Module Access]")
    result8 = bottom.bottom_func()
    print(f"bottom.bottom_func() = {result8}")
    assert result8 == "bottom", f"Expected 'bottom', got {result8}"
    result8b = mid_level.mid_func()
    print(f"mid_level.mid_func() = {result8b}")
    assert result8b == "mid_bottom", f"Expected 'mid_bottom', got {result8b}"
    result8c = top_level.top_func()
    print(f"top_level.top_func() = {result8c}")
    assert result8c == "top_mid_bottom", f"Expected 'top_mid_bottom', got {result8c}"
    print("PASS")

    # Test 9: Nested classes access
    print("\n[Test 9: Nested Class Instantiation]")
    obj = bottom.BottomClass()
    result9 = obj.method()
    print(f"BottomClass().method() = {result9}")
    assert result9 == "bottom_class_method", f"Expected 'bottom_class_method', got {result9}"
    print("PASS")

    # Test 10: Constants from nested modules
    print("\n[Test 10: Nested Module Constants]")
    result10 = bottom.BOTTOM_CONST
    print(f"bottom.BOTTOM_CONST = {result10}")
    assert result10 == 42, f"Expected 42, got {result10}"
    print("PASS")

    print("\n=== All Tests Passed ===")

run_all_tests()
