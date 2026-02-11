from imports.test_runner import run_all_tests as run_all_import_tests
from algorithm.test_runner import run_all_tests as run_all_algorithm_tests
from basic import test_runner
from planned.test_feature_roadmap import run_tests as run_planned_feature_tests

test_runner.run_all_tests()
run_planned_feature_tests()
run_all_algorithm_tests()
run_all_import_tests()
