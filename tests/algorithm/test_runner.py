from . import test_list_algorithms
from . import test_data_structures
from . import test_graph_algorithms
from . import test_math_algorithms


def run_all_tests() -> None:
    test_list_algorithms.run_tests()
    test_data_structures.run_tests()
    test_graph_algorithms.run_tests()
    test_math_algorithms.run_tests()
