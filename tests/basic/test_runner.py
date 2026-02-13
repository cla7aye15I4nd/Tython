from . import test_int
from . import test_float
from . import test_bool
from . import test_if_else
from . import test_while
from . import test_assert
from . import test_comparison
from . import test_arithmetic
from . import test_variables
from . import test_break_continue
from . import test_unary
from . import test_logical
from . import test_floor_div
from . import test_true_div
from . import test_pow
from . import test_bitwise
from . import test_augmented
from . import test_chained_cmp
from . import test_mixed_type
from . import test_builtins
from . import test_float_cmp
from . import test_casting
import basic.test_truthiness as test_truthiness
import basic.test_class as test_class
import basic.test_nested_class as test_nested_class
import basic.test_str as test_str
import basic.test_magic_str_repr as test_magic_str_repr
import basic.test_magic_len as test_magic_len
import basic.test_bytes as test_bytes
import basic.test_bytearray as test_bytearray
import basic.test_list as test_list
import basic.test_tuple as test_tuple
import basic.test_for as test_for
import basic.test_comprehension as test_comprehension
import basic.test_exception_iter_stress as test_exception_iter_stress
import basic.test_codegen_edges as test_codegen_edges
import basic.test_nested_function as test_nested_function
import basic.test_containment as test_containment
import basic.test_identity as test_identity
import basic.test_augmented_fields as test_augmented_fields
import basic.test_list_eq_complex as test_list_eq_complex
import basic.test_print_complex as test_print_complex
import basic.test_subscript_aug as test_subscript_aug
import basic.test_bytearray_compare as test_bytearray_compare
import basic.test_coverage_edges as test_coverage_edges
import basic.test_str_auto as test_str_auto
import basic.test_feature_roadmap as test_feature_roadmap
import basic.test_dict_set_annotations as test_dict_set_annotations
import basic.test_dict_set as test_dict_set
import basic.test_function_call_args as test_function_call_args
import basic.test_fstring as test_fstring
import basic.test_pass_ellipsis_docstring as test_pass_ellipsis_docstring


def run_all_tests() -> None:
    test_int.run_tests()
    test_float.run_tests()
    test_bool.run_tests()
    test_if_else.run_tests()
    test_while.run_tests()
    test_assert.run_tests()
    test_comparison.run_tests()
    test_arithmetic.run_tests()
    test_variables.run_tests()
    test_break_continue.run_tests()
    test_unary.run_tests()
    test_logical.run_tests()
    test_floor_div.run_tests()
    test_true_div.run_tests()
    test_pow.run_tests()
    test_bitwise.run_tests()
    test_augmented.run_tests()
    test_chained_cmp.run_tests()
    test_mixed_type.run_tests()
    test_builtins.run_tests()
    test_float_cmp.run_tests()
    test_casting.run_tests()
    test_truthiness.run_tests()
    test_class.run_tests()
    test_nested_class.run_tests()
    test_str.run_tests()
    test_magic_str_repr.run_tests()
    test_magic_len.run_tests()
    test_bytes.run_tests()
    test_bytearray.run_tests()
    test_list.run_tests()
    test_tuple.run_tests()
    test_for.run_tests()
    test_comprehension.run_tests()
    test_exception_iter_stress.run_tests()
    test_codegen_edges.run_tests()
    test_nested_function.run_tests()
    test_containment.run_tests()
    test_identity.run_tests()
    test_augmented_fields.run_tests()
    test_list_eq_complex.run_tests()
    test_print_complex.run_tests()
    test_subscript_aug.run_tests()
    test_bytearray_compare.run_tests()
    test_coverage_edges.run_tests()
    test_str_auto.run_tests()
    test_feature_roadmap.run_tests()
    test_dict_set_annotations.run_tests()
    test_dict_set.run_tests()
    test_function_call_args.run_tests()
    test_fstring.run_tests()
    test_pass_ellipsis_docstring.run_tests()
