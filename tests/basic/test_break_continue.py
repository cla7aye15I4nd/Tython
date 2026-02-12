def test_break_simple() -> None:
    i: int = 0
    while i < 10:
        if i == 5:
            break
        i = i + 1
    print(i)
    print('CHECK test_break_continue lhs:', i)
    print('CHECK test_break_continue rhs:', 5)
    assert i == 5


def test_continue_simple() -> None:
    total: int = 0
    i: int = 0
    while i < 10:
        i = i + 1
        if i % 2 == 0:
            continue
        total = total + i
    print(total)
    print('CHECK test_break_continue lhs:', total)
    print('CHECK test_break_continue rhs:', 25)
    assert total == 25


def test_break_nested_inner() -> None:
    total: int = 0
    i: int = 0
    while i < 3:
        j: int = 0
        while j < 10:
            if j == 2:
                break
            j = j + 1
            total = total + 1
        i = i + 1
    print(total)
    print('CHECK test_break_continue lhs:', total)
    print('CHECK test_break_continue rhs:', 6)
    assert total == 6


def test_continue_nested_inner() -> None:
    total: int = 0
    i: int = 0
    while i < 3:
        j: int = 0
        while j < 5:
            j = j + 1
            if j == 3:
                continue
            total = total + 1
        i = i + 1
    print(total)
    print('CHECK test_break_continue lhs:', total)
    print('CHECK test_break_continue rhs:', 12)
    assert total == 12


def test_break_first_iteration() -> None:
    i: int = 0
    while i < 100:
        break
    print(i)
    print('CHECK test_break_continue lhs:', i)
    print('CHECK test_break_continue rhs:', 0)
    assert i == 0


def test_continue_all_iterations() -> None:
    count: int = 0
    i: int = 0
    while i < 5:
        i = i + 1
        continue
        count = count + 1
    print(count)
    print('CHECK test_break_continue lhs:', count)
    print('CHECK test_break_continue rhs:', 0)
    assert count == 0


def test_break_with_accumulator() -> None:
    total: int = 0
    i: int = 1
    while i <= 100:
        total = total + i
        if total > 50:
            break
        i = i + 1
    print(total)
    print('CHECK test_break_continue lhs:', total)
    print('CHECK test_break_continue rhs:', 55)
    assert total == 55


def test_continue_skip_multiples() -> None:
    total: int = 0
    i: int = 0
    while i < 20:
        i = i + 1
        if i % 3 == 0:
            continue
        if i % 5 == 0:
            continue
        total = total + i
    print(total)
    print('CHECK test_break_continue lhs:', total)
    print('CHECK test_break_continue rhs:', 112)
    assert total == 112


def run_tests() -> None:
    test_break_simple()
    test_continue_simple()
    test_break_nested_inner()
    test_continue_nested_inner()
    test_break_first_iteration()
    test_continue_all_iterations()
    test_break_with_accumulator()
    test_continue_skip_multiples()
