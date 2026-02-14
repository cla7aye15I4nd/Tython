import random


def add3(a: int, b: int = 10, c: int = 100) -> int:
    return a + b + c

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


def run_tests() -> None:
    test_random_native_calls()
    test_keyword_binding_still_works()
