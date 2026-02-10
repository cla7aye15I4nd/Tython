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
