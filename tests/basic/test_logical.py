def test_and_both_truthy() -> None:
    x: int = 1 and 2
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 2)
    assert x == 2


def test_and_left_falsy() -> None:
    x: int = 0 and 2
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 0)
    assert x == 0


def test_or_left_truthy() -> None:
    x: int = 1 or 2
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 1)
    assert x == 1


def test_or_left_falsy() -> None:
    x: int = 0 or 2
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 2)
    assert x == 2


def test_and_bool() -> None:
    x: bool = True and False
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', False)
    assert x == False


def test_or_bool() -> None:
    x: bool = False or True
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', True)
    assert x == True


def test_and_chain() -> None:
    x: int = 1 and 2 and 3
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 3)
    assert x == 3


def test_or_chain() -> None:
    x: int = 0 or 0 or 5
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 5)
    assert x == 5


def test_short_circuit_and() -> None:
    x: int = 0 and 999
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 0)
    assert x == 0


def test_short_circuit_or() -> None:
    x: int = 1 or 999
    print('CHECK test_logical lhs:', x)
    print('CHECK test_logical rhs:', 1)
    assert x == 1


def run_tests() -> None:
    test_and_both_truthy()
    test_and_left_falsy()
    test_or_left_truthy()
    test_or_left_falsy()
    test_and_bool()
    test_or_bool()
    test_and_chain()
    test_or_chain()
    test_short_circuit_and()
    test_short_circuit_or()
