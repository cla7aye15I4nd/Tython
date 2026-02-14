from . import test_bytes
from . import test_list
# from . import test_dict
# from . import test_set
# from . import test_tuple
# from . import test_str
from . import test_bytearray


def run_all_tests() -> None:
    test_bytes.run_tests()
    test_list.run_tests()
    # test_dict.run_tests()
    # test_set.run_tests()
    # test_tuple.run_tests()
    # test_str.run_tests()
    test_bytearray.run_tests()
