def test_if_int_truthy() -> None:
    result: int = 0
    if 42:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 1)
    assert result == 1


def test_if_int_falsy() -> None:
    result: int = +0
    if 0:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 0)
    assert result == 0


def test_if_negative_int_truthy() -> None:
    result: int = 0
    if -5:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 1)
    assert result == 1
    return


def test_if_float_truthy() -> None:
    result: int = 0
    if 1.0:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 1)
    assert result == 1


def test_if_float_falsy() -> None:
    result: int = 0
    if 0.0:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 0)
    assert result == 0


def test_if_bool_truthy() -> None:
    result: int = 0
    if True:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 1)
    assert result == 1


def test_if_bool_falsy() -> None:
    result: int = 0
    if False:
        result = 1
    print('CHECK test_truthiness lhs:', result)
    print('CHECK test_truthiness rhs:', 0)
    assert result == 0


def test_while_int_countdown() -> None:
    x: int = 5
    count: int = 0
    while x:
        count += 1
        x -= 1
    print('CHECK test_truthiness lhs:', count)
    print('CHECK test_truthiness rhs:', 5)
    assert count == 5
    print('CHECK test_truthiness lhs:', x)
    print('CHECK test_truthiness rhs:', 0)
    assert x == 0


def test_assert_int_nonzero() -> None:
    print('CHECK test_truthiness assert expr:', '1')
    assert 1
    print('CHECK test_truthiness assert expr:', '42')
    assert 42
    print('CHECK test_truthiness assert expr:', '-1')
    assert -1


def test_assert_float_nonzero() -> None:
    print('CHECK test_truthiness assert expr:', '1.0')
    assert 1.0
    print('CHECK test_truthiness assert expr:', '99.9')
    assert 99.9


def test_assert_bool_true() -> None:
    print('CHECK test_truthiness assert expr:', 'True')
    assert True


def test_assert_expression() -> None:
    x: int = 10
    print('CHECK test_truthiness assert expr:', 'x')
    assert x


class TruthyBox:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def test_truthiness_sequences_and_class() -> None:
    out: int = 0
    d: dict[int, int] = {1: 2}
    s: set[int] = {1, 2}
    empty_tuple: tuple[()] = ()
    non_empty_tuple: tuple[int] = (1,)
    if "abc":
        out += 1
    if b"ab":
        out += 1
    if bytearray(b"ab"):
        out += 1
    if [1, 2]:
        out += 1
    if d:
        out += 1
    if s:
        out += 1
    if empty_tuple:
        out += 100
    if non_empty_tuple:
        out += 1
    if TruthyBox(7):
        out += 1
    print('CHECK test_truthiness lhs:', out)
    print('CHECK test_truthiness rhs:', 8)
    assert out == 8


def run_tests() -> None:
    test_if_int_truthy()
    test_if_int_falsy()
    test_if_negative_int_truthy()
    test_if_float_truthy()
    test_if_float_falsy()
    test_if_bool_truthy()
    test_if_bool_falsy()
    test_while_int_countdown()
    test_assert_int_nonzero()
    test_assert_float_nonzero()
    test_assert_bool_true()
    test_assert_expression()
    test_truthiness_sequences_and_class()
