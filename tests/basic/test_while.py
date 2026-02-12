def test_while_count_up() -> None:
    i: int = 0
    while i < 5:
        i = i + 1
    print(i)
    print('CHECK test_while lhs:', i)
    print('CHECK test_while rhs:', 5)
    assert i == 5


def test_while_count_down() -> None:
    i: int = 10
    while i > 0:
        i = i - 1
    print(i)
    print('CHECK test_while lhs:', i)
    print('CHECK test_while rhs:', 0)
    assert i == 0


def test_while_sum() -> None:
    i: int = 1
    total: int = 0
    while i <= 10:
        total = total + i
        i = i + 1
    print(total)
    print('CHECK test_while lhs:', total)
    print('CHECK test_while rhs:', 55)
    assert total == 55


def test_while_factorial() -> None:
    n: int = 6
    result: int = 1
    i: int = 1
    while i <= n:
        result = result * i
        i = i + 1
    print(result)
    print('CHECK test_while lhs:', result)
    print('CHECK test_while rhs:', 720)
    assert result == 720


def test_while_power() -> None:
    base: int = 2
    exp: int = 10
    result: int = 1
    i: int = 0
    while i < exp:
        result = result * base
        i = i + 1
    print(result)
    print('CHECK test_while lhs:', result)
    print('CHECK test_while rhs:', 1024)
    assert result == 1024


def test_while_no_iteration() -> None:
    i: int = 10
    count: int = 0
    while i < 5:
        count = count + 1
        i = i + 1
    print(count)
    print('CHECK test_while lhs:', count)
    print('CHECK test_while rhs:', 0)
    assert count == 0


def test_while_single_iteration() -> None:
    i: int = 4
    count: int = 0
    while i < 5:
        count = count + 1
        i = i + 1
    print(count)
    print('CHECK test_while lhs:', count)
    print('CHECK test_while rhs:', 1)
    assert count == 1


def test_while_nested() -> None:
    total: int = 0
    i: int = 0
    while i < 3:
        j: int = 0
        while j < 3:
            total = total + 1
            j = j + 1
        i = i + 1
    print(total)
    print('CHECK test_while lhs:', total)
    print('CHECK test_while rhs:', 9)
    assert total == 9


def test_while_nested_multiply() -> None:
    total: int = 0
    i: int = 1
    while i <= 3:
        j: int = 1
        while j <= 3:
            total = total + i * j
            j = j + 1
        i = i + 1
    print(total)
    print('CHECK test_while lhs:', total)
    print('CHECK test_while rhs:', 36)
    assert total == 36


def test_while_with_condition() -> None:
    i: int = 1
    count: int = 0
    while i * i <= 100:
        count = count + 1
        i = i + 1
    print(count)
    print('CHECK test_while lhs:', count)
    print('CHECK test_while rhs:', 10)
    assert count == 10


def test_while_fibonacci() -> None:
    a: int = 0
    b: int = 1
    i: int = 0
    while i < 10:
        temp: int = b
        b = a + b
        a = temp
        i = i + 1
    print(a)
    print('CHECK test_while lhs:', a)
    print('CHECK test_while rhs:', 55)
    assert a == 55


def test_while_gcd() -> None:
    a: int = 48
    b: int = 18
    while b != 0:
        temp: int = b
        b = a % b
        a = temp
    print(a)
    print('CHECK test_while lhs:', a)
    print('CHECK test_while rhs:', 6)
    assert a == 6


def run_tests() -> None:
    test_while_count_up()
    test_while_count_down()
    test_while_sum()
    test_while_factorial()
    test_while_power()
    test_while_no_iteration()
    test_while_single_iteration()
    test_while_nested()
    test_while_nested_multiply()
    test_while_with_condition()
    test_while_fibonacci()
    test_while_gcd()
