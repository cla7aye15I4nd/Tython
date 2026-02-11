def test_for_range_stop_only() -> None:
    total: int = 0
    for i in range(5):
        total = total + i
    print(total)
    assert total == 10


def test_for_range_start_stop() -> None:
    total: int = 0
    for i in range(2, 7):
        total = total + i
    print(total)
    assert total == 20


def test_for_range_start_stop_step() -> None:
    total: int = 0
    for i in range(1, 10, 3):
        total = total + i
    print(total)
    assert total == 12


def test_for_range_negative_step() -> None:
    total: int = 0
    for i in range(5, 0, -2):
        total = total + i
    print(total)
    assert total == 9


def test_for_continue() -> None:
    total: int = 0
    for i in range(1, 8):
        if i % 2 == 0:
            continue
        total = total + i
    print(total)
    assert total == 16


def test_for_break() -> None:
    total: int = 0
    for i in range(10):
        if i == 4:
            break
        total = total + i
    print(total)
    assert total == 6


def test_for_range_args_evaluated_once() -> None:
    stop: int = 5
    count: int = 0
    for i in range(0, stop):
        stop = 1
        count = count + 1
    print(count)
    assert count == 5


def test_for_range_step_minus_one() -> None:
    total: int = 0
    for i in range(7, -1, -1):
        total = total + i
    print(total)
    assert total == 28


def test_for_range_step_minus_two_mixed_sign() -> None:
    total: int = 0
    for i in range(9, -4, -2):
        total = total + i
    print(total)
    assert total == 21


def test_for_nested_negative_steps() -> None:
    total: int = 0
    for i in range(5, -1, -1):
        for j in range(6, 0, -2):
            total = total + i * j
    print(total)
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
    assert total == 41


def test_for_negative_empty_ranges() -> None:
    count: int = 0
    for _ in range(0, 5, -1):
        count = count + 1
    for _ in range(3, 3, -2):
        count = count + 1
    print(count)
    assert count == 0


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
