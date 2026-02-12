def add(a: int, b: int) -> int:
    return a + b


def apply_binop(f: 'callable[[int, int], int]', x: int, y: int) -> int:
    return f(x, y)


def test_callable_as_parameter() -> None:
    result: int = apply_binop(add, 5, 9)
    print(result)
    print('CHECK test_callable lhs:', result)
    print('CHECK test_callable rhs:', 14)
    assert result == 14


def test_callable_variable_annotation() -> None:
    op: "callable[[int, int], int]" = add
    result: int = apply_binop(op, 10, 3)
    print(result)
    print('CHECK test_callable lhs:', result)
    print('CHECK test_callable rhs:', 13)
    assert result == 13


def run_tests() -> None:
    test_callable_as_parameter()
    test_callable_variable_annotation()
