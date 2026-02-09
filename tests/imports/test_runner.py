"""
Test runner module for Tython integration tests.
Contains all test logic for various import patterns and module dependencies.
"""

# Test various import patterns - simple sibling imports
from . import module_a
from . import module_b
from . import module_c
from . import module_d

# Complex nested imports
from . import module_e
from . import module_f
from . import module_g

# Direct nested module imports
from .nested.deep.deeper import bottom
from .nested.deep.deeper import complex_imports
from .nested.deep import mid_level
from .nested.deep import cross_level_imports
from .nested import top_level

# Import new module H that tests deep access
from . import module_h


def test_module_a():
    """Test 1: Simple module import (no dependencies)"""
    print("\n[Test 1: Module A - No Dependencies]")
    result1 = module_a.func_a()
    print(f"module_a.func_a() = {result1}")
    assert result1 == "A", f"Expected 'A', got {result1}"
    result1b = module_a.compute_a(10)
    print(f"module_a.compute_a(10) = {result1b}")
    assert result1b == 11, f"Expected 11, got {result1b}"
    print("PASS")


def test_module_b():
    """Test 2: Sibling import (B imports A)"""
    print("\n[Test 2: Module B - Imports Sibling A]")
    result2 = module_b.func_b()
    print(f"module_b.func_b() = {result2}")
    assert result2 == "BA", f"Expected 'BA', got {result2}"
    result2b = module_b.compute_b(5)
    print(f"module_b.compute_b(5) = {result2b}")
    assert result2b == 12, f"Expected 12, got {result2b}"
    print("PASS")


def test_module_c():
    """Test 3: Multiple sibling imports (C imports A and B)"""
    print("\n[Test 3: Module C - Imports Siblings A and B]")
    result3 = module_c.func_c()
    print(f"module_c.func_c() = {result3}")
    assert result3 == "CABA", f"Expected 'CABA', got {result3}"
    result3b = module_c.compute_c(3)
    print(f"module_c.compute_c(3) = {result3b}")
    assert result3b == 12, f"Expected 12, got {result3b}"
    print("PASS")


def test_module_d():
    """Test 4: Deep dependency chain (D imports C, which imports B, which imports A)"""
    print("\n[Test 4: Module D - Deep Dependency Chain]")
    result4 = module_d.func_d()
    print(f"module_d.func_d() = {result4}")
    assert result4 == "DCABA", f"Expected 'DCABA', got {result4}"
    result4b = module_d.compute_d(2)
    print(f"module_d.compute_d(2) = {result4b}")
    assert result4b == 12, f"Expected 12, got {result4b}"
    print("PASS")


def test_module_e():
    """Test 5: Nested package imports (from nested.deep.deeper.bottom import X)"""
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


def test_module_f():
    """Test 6: Full nested module path imports (import nested.deep.deeper.bottom)"""
    print("\n[Test 6: Module F - Full Nested Module Paths]")
    result6 = module_f.module_f_func()
    print(f"module_f.module_f_func() = {result6}")
    assert result6 == "F_bottom_mid_bottom_top_mid_bottom", f"Expected 'F_bottom_mid_bottom_top_mid_bottom', got {result6}"
    result6b = module_f.module_f_access()
    print(f"module_f.module_f_access() = {result6b}")
    assert result6b == 126, f"Expected 126, got {result6b}"
    print("PASS")


def test_module_g():
    """Test 7: Mixed import styles with aliases"""
    print("\n[Test 7: Module G - Mixed Import Styles]")
    result7 = module_g.module_g_func()
    print(f"module_g.module_g_func() = {result7}")
    expected7 = "G_sibling_b_top_mid_bottom_sibling_a_calls_top_mid_bottom_sibling_a_calls_top_mid_bottom"
    assert result7 == expected7, f"Expected '{expected7}', got {result7}"
    result7b = module_g.module_g_use_e()
    print(f"module_g.module_g_use_e() = {result7b}")
    assert result7b == "E_bottom_mid_bottom_top_mid_bottom", f"Expected 'E_bottom_mid_bottom_top_mid_bottom', got {result7b}"
    print("PASS")


def test_direct_nested_access():
    """Test 8: Direct access to deeply nested modules"""
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


def test_nested_class_instantiation():
    """Test 9: Nested classes access"""
    print("\n[Test 9: Nested Class Instantiation]")
    obj = bottom.BottomClass()
    result9 = obj.method()
    print(f"BottomClass().method() = {result9}")
    assert result9 == "bottom_class_method", f"Expected 'bottom_class_method', got {result9}"
    print("PASS")


def test_nested_module_constants():
    """Test 10: Nested module constants"""
    print("\n[Test 10: Nested Module Constants]")
    result10 = bottom.BOTTOM_CONST
    print(f"bottom.BOTTOM_CONST = {result10}")
    assert result10 == 42, f"Expected 42, got {result10}"
    print("PASS")


def test_alternative_import_syntax():
    """Test 11: Alternative import syntax (import X as Y)"""
    print("\n[Test 11: Alternative Import Syntax]")
    # Test aliased imports
    from . import module_a as mod_a
    result11 = mod_a.func_a()
    print(f"mod_a.func_a() = {result11}")
    assert result11 == "A", f"Expected 'A', got {result11}"
    print("PASS")


def test_from_import_star():
    """Test 12: Test specific function imports"""
    print("\n[Test 12: Specific Function Imports]")
    # Import specific functions from modules
    from .module_a import func_a, compute_a
    result12a = func_a()
    print(f"func_a() = {result12a}")
    assert result12a == "A", f"Expected 'A', got {result12a}"
    result12b = compute_a(20)
    print(f"compute_a(20) = {result12b}")
    assert result12b == 21, f"Expected 21, got {result12b}"
    print("PASS")


def test_nested_from_imports():
    """Test 13: From imports with nested paths"""
    print("\n[Test 13: From Imports with Nested Paths]")
    from .nested.deep.deeper.bottom import bottom_func, BottomClass, BOTTOM_CONST
    result13a = bottom_func()
    print(f"bottom_func() = {result13a}")
    assert result13a == "bottom", f"Expected 'bottom', got {result13a}"

    obj = BottomClass()
    result13b = obj.method()
    print(f"BottomClass().method() = {result13b}")
    assert result13b == "bottom_class_method", f"Expected 'bottom_class_method', got {result13b}"

    print(f"BOTTOM_CONST = {BOTTOM_CONST}")
    assert BOTTOM_CONST == 42, f"Expected 42, got {BOTTOM_CONST}"
    print("PASS")


def test_complex_relative_imports():
    """Test 14: Complex relative imports with multiple parent traversals"""
    print("\n[Test 14: Complex Relative Imports - Multiple Parent Levels]")

    # Test single dot (same directory)
    result14a = complex_imports.test_same_dir()
    print(f"complex_imports.test_same_dir() = {result14a}")
    assert result14a == "bottom", f"Expected 'bottom', got {result14a}"

    # Test double dot (parent directory ..)
    result14b = complex_imports.test_parent_import()
    print(f"complex_imports.test_parent_import() = {result14b}")
    assert result14b == "mid_bottom", f"Expected 'mid_bottom', got {result14b}"

    # Test triple dot (grandparent directory ...)
    result14c = complex_imports.test_grandparent_import()
    print(f"complex_imports.test_grandparent_import() = {result14c}")
    assert result14c == "top_mid_bottom", f"Expected 'top_mid_bottom', got {result14c}"

    # Test quadruple dot (great-grandparent directory ....)
    result14d = complex_imports.test_great_grandparent_import()
    print(f"complex_imports.test_great_grandparent_import() = {result14d}")
    assert result14d == "complex_A_BA", f"Expected 'complex_A_BA', got {result14d}"

    print("PASS")


def test_specific_function_relative_imports():
    """Test 15: Specific function imports from multiple levels"""
    print("\n[Test 15: Specific Function Imports Across Levels]")

    result15 = complex_imports.test_specific_imports()
    print(f"complex_imports.test_specific_imports() = {result15}")
    expected15 = "specific_CABA_top_mid_bottom_mid_bottom_bottom"
    assert result15 == expected15, f"Expected '{expected15}', got {result15}"

    result15b = complex_imports.test_class_import()
    print(f"complex_imports.test_class_import() = {result15b}")
    assert result15b == "bottom_class_method", f"Expected 'bottom_class_method', got {result15b}"

    print("PASS")


def test_cross_level_imports():
    """Test 16: Cross-level imports from intermediate depth"""
    print("\n[Test 16: Cross-Level Imports from Mid-Depth]")

    result16a = cross_level_imports.cross_level_func()
    print(f"cross_level_imports.cross_level_func() = {result16a}")
    assert result16a == "cross_bottom_mid_bottom_top_mid_bottom_A", f"Expected 'cross_bottom_mid_bottom_top_mid_bottom_A', got {result16a}"

    result16b = cross_level_imports.test_complex_imports_module()
    print(f"cross_level_imports.test_complex_imports_module() = {result16b}")
    assert result16b == "bottom", f"Expected 'bottom', got {result16b}"

    result16c = cross_level_imports.test_sibling_interaction()
    print(f"cross_level_imports.test_sibling_interaction() = {result16c}")
    assert result16c == "siblings_sibling_a_calls_top_mid_bottom_sibling_b_top_mid_bottom", f"Expected 'siblings_sibling_a_calls_top_mid_bottom_sibling_b_top_mid_bottom', got {result16c}"

    print("PASS")


def test_deep_chain_imports():
    """Test 17: Chained function calls across deep nesting"""
    print("\n[Test 17: Deep Chain Imports]")

    result17a = complex_imports.complex_chain()
    print(f"complex_imports.complex_chain() = {result17a}")
    assert result17a == "chain_bottom_mid_bottom_top_mid_bottom", f"Expected 'chain_bottom_mid_bottom_top_mid_bottom', got {result17a}"

    result17b = cross_level_imports.test_deep_chain()
    print(f"cross_level_imports.test_deep_chain() = {result17b}")
    assert result17b == "chain_bottom_mid_bottom_top_mid_bottom", f"Expected 'chain_bottom_mid_bottom_top_mid_bottom', got {result17b}"

    print("PASS")


def test_module_h_deep_access():
    """Test 18: Top-level module accessing deeply nested modules"""
    print("\n[Test 18: Module H - Deep Access from Top Level]")

    result18a = module_h.module_h_deep_access()
    print(f"module_h.module_h_deep_access() = {result18a}")
    assert result18a == "H_bottom_bottom_cross_bottom_mid_bottom_top_mid_bottom_A", f"Expected 'H_bottom_bottom_cross_bottom_mid_bottom_top_mid_bottom_A', got {result18a}"

    result18b = module_h.module_h_nested_chain()
    print(f"module_h.module_h_nested_chain() = {result18b}")
    assert result18b == "H_chain_mid_bottom_top_mid_bottom", f"Expected 'H_chain_mid_bottom_top_mid_bottom', got {result18b}"

    print("PASS")


def test_module_h_complex_patterns():
    """Test 19: Module H using complex import patterns"""
    print("\n[Test 19: Module H - Complex Import Patterns]")

    result19a = module_h.module_h_complex_chain()
    print(f"module_h.module_h_complex_chain() = {result19a}")
    assert result19a == "H_bottom_mid_bottom_top_mid_bottom_complex_A_BA", f"Expected 'H_bottom_mid_bottom_top_mid_bottom_complex_A_BA', got {result19a}"

    result19b = module_h.module_h_cross_level()
    print(f"module_h.module_h_cross_level() = {result19b}")
    expected19b = "H_cross_cross_bottom_mid_bottom_top_mid_bottom_A_siblings_sibling_a_calls_top_mid_bottom_sibling_b_top_mid_bottom"
    assert result19b == expected19b, f"Expected '{expected19b}', got {result19b}"

    print("PASS")


def test_renamed_imports():
    """Test 20: Renamed imports and aliases across levels"""
    print("\n[Test 20: Renamed Imports and Aliases]")

    result20 = cross_level_imports.test_renamed_import()
    print(f"cross_level_imports.test_renamed_import() = {result20}")
    assert result20 == "top_mid_bottom", f"Expected 'top_mid_bottom', got {result20}"

    print("PASS")


def test_compute_across_levels():
    """Test 21: Computation functions across import levels"""
    print("\n[Test 21: Compute Across Import Levels]")

    # cross_level_imports.compute_cross(5):
    # 1. a_result = module_a.compute_a(5) = 6
    # 2. d_result = module_d.compute_d(6)
    #    - val = module_a.compute_a(6) = 7
    #    - return module_c.compute_c(7) = (7+1) + (7+1)*2 = 8 + 16 = 24
    result21a = cross_level_imports.compute_cross(5)
    print(f"cross_level_imports.compute_cross(5) = {result21a}")
    assert result21a == 24, f"Expected 24, got {result21a}"

    # module_h.module_h_compute(10):
    # 1. a_result = module_a.compute_a(10) = 11
    # 2. d_result = module_d.compute_d(11)
    #    - val = module_a.compute_a(11) = 12
    #    - return module_c.compute_c(12) = (12+1) + (12+1)*2 = 13 + 26 = 39
    result21b = module_h.module_h_compute(10)
    print(f"module_h.module_h_compute(10) = {result21b}")
    assert result21b == 39, f"Expected 39, got {result21b}"

    print("PASS")


def run_all_tests():
    """Run all integration tests for Tython import system."""
    print("=== Running Tython Import Tests ===")

    # Basic import tests
    test_module_a()
    test_module_b()
    test_module_c()
    test_module_d()
    test_module_e()
    test_module_f()
    test_module_g()

    # Direct nested access tests
    test_direct_nested_access()
    test_nested_class_instantiation()
    test_nested_module_constants()

    # Alternative syntax tests
    test_alternative_import_syntax()
    test_from_import_star()
    test_nested_from_imports()

    # Complex relative import tests (. .. ... ....)
    test_complex_relative_imports()
    test_specific_function_relative_imports()

    # Cross-level and deep nesting tests
    test_cross_level_imports()
    test_deep_chain_imports()

    # Top-level accessing deeply nested modules
    test_module_h_deep_access()
    test_module_h_complex_patterns()

    # Advanced import patterns
    test_renamed_imports()
    test_compute_across_levels()

    print("\n=== All Tests Passed ===")
