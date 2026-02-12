from . import test_magic_ops
from . import test_magic_ops_complex
from . import test_microgpt_value_magic


def run_all_tests() -> None:
    test_magic_ops.run_tests()
    test_magic_ops_complex.run_tests()
    test_microgpt_value_magic.run_tests()
