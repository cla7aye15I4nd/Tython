import random
import math


def add3(a: int, b: int = 10, c: int = 100) -> int:
    return a + b + c


class Acc:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __add__(self, other: "Acc") -> "Acc":
        return Acc(self.value + other.value)

def test_random_native_calls() -> None:
    random.seed(1337)
    g1: float = random.gauss(0.0, 1.0)
    random.seed(1337)
    g2: float = random.gauss(0.0, 1.0)
    same: bool = g1 == g2
    print('CHECK test_call_paths lhs:', same)
    print('CHECK test_call_paths rhs:', True)
    assert same

    xs: list[int] = [1, 2, 3, 4]
    random.shuffle(xs)
    print('CHECK test_call_paths lhs:', len(xs))
    print('CHECK test_call_paths rhs:', 4)
    assert len(xs) == 4
    print('CHECK test_call_paths lhs:', sum(xs))
    print('CHECK test_call_paths rhs:', 10)
    assert sum(xs) == 10

    picks: list[int] = random.choices([1, 2, 3], weights=[0.1, 0.2, 0.7])
    print('CHECK test_call_paths lhs:', len(picks))
    print('CHECK test_call_paths rhs:', 1)
    assert len(picks) == 1
    print('CHECK test_call_paths assert expr:', 'picks[0] >= 1')
    assert picks[0] >= 1
    print('CHECK test_call_paths assert expr:', 'picks[0] <= 3')
    assert picks[0] <= 3


def test_keyword_binding_still_works() -> None:
    print('CHECK test_call_paths lhs:', add3(1, c=5))
    print('CHECK test_call_paths rhs:', 16)
    assert add3(1, c=5) == 16

def test_math_native_calls() -> None:
    log_one: float = math.log(1.0)
    exp_zero: float = math.exp(0.0)
    print('CHECK test_call_paths lhs:', log_one)
    print('CHECK test_call_paths rhs:', 0.0)
    assert log_one == 0.0
    print('CHECK test_call_paths lhs:', exp_zero)
    print('CHECK test_call_paths rhs:', 1.0)
    assert exp_zero == 1.0

def test_sum_generator_fast_path() -> None:
    total: int = sum((i for i in [1, 2, 3, 4]), 10)
    print('CHECK test_call_paths lhs:', total)
    print('CHECK test_call_paths rhs:', 20)
    assert total == 20


def test_sum_class_list_special_case() -> None:
    total: Acc = sum([Acc(1), Acc(2), Acc(3)], Acc(10))
    print('CHECK test_call_paths lhs:', total.value)
    print('CHECK test_call_paths rhs:', 16)
    assert total.value == 16


def run_tests() -> None:
    test_random_native_calls()
    test_keyword_binding_still_works()
    test_math_native_calls()
    test_sum_generator_fast_path()
    test_sum_class_list_special_case()
