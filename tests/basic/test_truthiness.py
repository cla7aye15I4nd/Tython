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
