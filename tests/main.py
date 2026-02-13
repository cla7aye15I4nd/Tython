from imports.test_runner import run_all_tests as run_all_import_tests
from algorithm.test_runner import run_all_tests as run_all_algorithm_tests
from classes.test_runner import run_all_tests as run_all_class_tests
from basic import test_runner

if __name__ == "__main__":
    test_runner.run_all_tests()
    run_all_class_tests()
    run_all_algorithm_tests()
    run_all_import_tests()
