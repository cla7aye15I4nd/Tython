def test_for_range_stop_only() -> None:
    total: int = 0
    for i in range(5):
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 10)
    assert total == 10


def test_for_range_start_stop() -> None:
    total: int = 0
    for i in range(2, 7):
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 20)
    assert total == 20


def test_for_range_start_stop_step() -> None:
    total: int = 0
    for i in range(1, 10, 3):
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 12)
    assert total == 12


def test_for_range_negative_step() -> None:
    total: int = 0
    for i in range(5, 0, -2):
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 9)
    assert total == 9


def test_for_continue() -> None:
    total: int = 0
    for i in range(1, 8):
        if i % 2 == 0:
            continue
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 16)
    assert total == 16


def test_for_break() -> None:
    total: int = 0
    for i in range(10):
        if i == 4:
            break
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 6)
    assert total == 6


def test_for_range_args_evaluated_once() -> None:
    stop: int = 5
    count: int = 0
    for i in range(0, stop):
        stop = 1
        count = count + 1
    print(count)
    print('CHECK test_for lhs:', count)
    print('CHECK test_for rhs:', 5)
    assert count == 5


def test_for_range_step_minus_one() -> None:
    total: int = 0
    for i in range(7, -1, -1):
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 28)
    assert total == 28


def test_for_range_step_minus_two_mixed_sign() -> None:
    total: int = 0
    for i in range(9, -4, -2):
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 21)
    assert total == 21


def test_for_nested_negative_steps() -> None:
    total: int = 0
    for i in range(5, -1, -1):
        for j in range(6, 0, -2):
            total = total + i * j
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 180)
    assert total == 180


def test_for_negative_step_with_continue_and_break() -> None:
    total: int = 0
    for i in range(10, -1, -1):
        if i == 8:
            continue
        if i == 3:
            break
        total = total + i
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 41)
    assert total == 41


def test_for_negative_empty_ranges() -> None:
    count: int = 0
    for _ in range(0, 5, -1):
        count = count + 1
    for _ in range(3, 3, -2):
        count = count + 1
    print(count)
    print('CHECK test_for lhs:', count)
    print('CHECK test_for rhs:', 0)
    assert count == 0


def test_for_tuple_iteration() -> None:
    values: tuple[int, int, int, int] = (2, 4, 6, 8)
    total: int = 0
    for n in values:
        total = total + n
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 20)
    assert total == 20


def test_for_tuple_break_continue() -> None:
    values: tuple[int, int, int, int, int] = (1, 2, 3, 4, 5)
    total: int = 0
    for n in values:
        if n == 2:
            continue
        if n == 5:
            break
        total = total + n
    print(total)
    print('CHECK test_for lhs:', total)
    print('CHECK test_for rhs:', 8)
    assert total == 8


def run_tests() -> None:
    test_for_range_stop_only()
    test_for_range_start_stop()
    test_for_range_start_stop_step()
    test_for_range_negative_step()
    test_for_continue()
    test_for_break()
    test_for_range_args_evaluated_once()
    test_for_range_step_minus_one()
    test_for_range_step_minus_two_mixed_sign()
    test_for_nested_negative_steps()
    test_for_negative_step_with_continue_and_break()
    test_for_negative_empty_ranges()
    test_for_tuple_iteration()
    test_for_tuple_break_continue()
