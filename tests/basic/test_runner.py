from . import test_int
from . import test_float
from . import test_bool
from . import test_if_else
from . import test_while
from . import test_assert
from . import test_comparison
from . import test_arithmetic
from . import test_variables


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
