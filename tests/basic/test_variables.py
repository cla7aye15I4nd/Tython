def test_simple_assignment() -> None:
    x: int = 42
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 42)
    assert x == 42


def test_annotated_int() -> None:
    x: int = 10
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 10)
    assert x == 10


def test_annotated_bool() -> None:
    x: bool = True
    result: int = int(x)
    print(result)
    print('CHECK test_variables lhs:', result)
    print('CHECK test_variables rhs:', 1)
    assert result == 1


def test_reassignment() -> None:
    x: int = 5
    x = 10
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 10)
    assert x == 10


def test_multiple_reassignment() -> None:
    x: int = 1
    x = 2
    x = 3
    x = 4
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 4)
    assert x == 4


def test_assignment_from_expression() -> None:
    a: int = 3
    b: int = 4
    c: int = a + b
    print(c)
    print('CHECK test_variables lhs:', c)
    print('CHECK test_variables rhs:', 7)
    assert c == 7


def test_self_referencing_update() -> None:
    x: int = 1
    x = x + 1
    x = x + 1
    x = x + 1
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 4)
    assert x == 4


def test_swap_with_temp() -> None:
    a: int = 10
    b: int = 20
    temp: int = a
    a = b
    b = temp
    print(a)
    print('CHECK test_variables lhs:', a)
    print('CHECK test_variables rhs:', 20)
    assert a == 20
    print(b)
    print('CHECK test_variables lhs:', b)
    print('CHECK test_variables rhs:', 10)
    assert b == 10


def test_variable_in_if_scope() -> None:
    x: int = 5
    if x == 5:
        x = 100
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 100)
    assert x == 100


def test_variable_in_else_scope() -> None:
    x: int = 5
    if x == 10:
        x = 100
    else:
        x = 200
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 200)
    assert x == 200


def test_variable_in_while_scope() -> None:
    x: int = 0
    i: int = 0
    while i < 5:
        x = x + 10
        i = i + 1
    print(x)
    print('CHECK test_variables lhs:', x)
    print('CHECK test_variables rhs:', 50)
    assert x == 50


def test_multiple_variables() -> None:
    a: int = 1
    b: int = 2
    c: int = 3
    d: int = 4
    result: int = a + b + c + d
    print(result)
    print('CHECK test_variables lhs:', result)
    print('CHECK test_variables rhs:', 10)
    assert result == 10


def test_variable_chain_computation() -> None:
    a: int = 2
    b: int = a * 3
    c: int = b + a
    d: int = c * 2
    print(d)
    print('CHECK test_variables lhs:', d)
    print('CHECK test_variables rhs:', 16)
    assert d == 16


def run_tests() -> None:
    test_simple_assignment()
    test_annotated_int()
    test_annotated_bool()
    test_reassignment()
    test_multiple_reassignment()
    test_assignment_from_expression()
    test_self_referencing_update()
    test_swap_with_temp()
    test_variable_in_if_scope()
    test_variable_in_else_scope()
    test_variable_in_while_scope()
    test_multiple_variables()
    test_variable_chain_computation()
