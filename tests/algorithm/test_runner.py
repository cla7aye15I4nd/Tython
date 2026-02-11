from . import test_list_algorithms
from . import test_data_structures
from . import test_graph_algorithms
from . import test_math_algorithms
from . import test_algorithm
from . import test_tree_algorithms
from . import test_linear_data_structures
from . import test_range_data_structures
from . import test_advanced_data_structures
from . import test_balanced_tree_structures
from . import test_skiplist_bloom
from . import test_interval_and_2d_fenwick
from . import test_monotonic_structures
from . import test_ordered_multiset_rollback_dsu
from . import test_persistent_data_structures


def run_all_tests() -> None:
    test_list_algorithms.run_tests()
    test_data_structures.run_tests()
    test_graph_algorithms.run_tests()
    test_math_algorithms.run_tests()
    test_algorithm.run_tests()
    test_tree_algorithms.run_tests()
    test_linear_data_structures.run_tests()
    test_range_data_structures.run_tests()
    test_advanced_data_structures.run_tests()
    test_balanced_tree_structures.run_tests()
    test_skiplist_bloom.run_tests()
    test_interval_and_2d_fenwick.run_tests()
    test_monotonic_structures.run_tests()
    test_ordered_multiset_rollback_dsu.run_tests()
    test_persistent_data_structures.run_tests()
