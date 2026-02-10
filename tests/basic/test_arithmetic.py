def test_precedence_mul_before_add() -> None:
    result: int = 2 + 3 * 4
    print(result)
    assert result == 14


def test_precedence_parentheses() -> None:
    result: int = (2 + 3) * 4
    print(result)
    assert result == 20


def test_precedence_nested_parens() -> None:
    result: int = ((1 + 2) * (3 + 4))
    print(result)
    assert result == 21


def test_precedence_sub_mul() -> None:
    result: int = 10 - 2 * 3
    print(result)
    assert result == 4


def test_precedence_mod() -> None:
    result: int = 10 + 7 % 3
    print(result)
    assert result == 11


def test_complex_expression() -> None:
    result: int = (5 + 3) * 2 - 4
    print(result)
    assert result == 12


def test_multi_term() -> None:
    result: int = 1 * 2 + 3 * 4 + 5 * 6
    print(result)
    assert result == 44


def test_subtraction_chain() -> None:
    result: int = 100 - 20 - 30 - 10
    print(result)
    assert result == 40


def test_mixed_operations() -> None:
    a: int = 10
    b: int = 3
    c: int = 7
    result: int = a * b + c
    print(result)
    assert result == 37


def test_square() -> None:
    x: int = 12
    result: int = x * x
    print(result)
    assert result == 144


def test_cube() -> None:
    x: int = 5
    result: int = x * x * x
    print(result)
    assert result == 125


def test_mod_pattern() -> None:
    x: int = 25
    result: int = x % 10
    print(result)
    assert result == 5


def test_mod_zero_result() -> None:
    x: int = 100
    result: int = x % 25
    print(result)
    assert result == 0


def test_arithmetic_with_negative() -> None:
    a: int = 0 - 5
    b: int = 3
    result: int = a + b
    print(result)
    assert result == 0 - 2


def test_double_negative() -> None:
    a: int = 0 - 3
    b: int = 0 - 7
    result: int = a + b
    print(result)
    assert result == 0 - 10


def test_negative_mul() -> None:
    a: int = 0 - 4
    b: int = 5
    result: int = a * b
    print(result)
    assert result == 0 - 20


def test_negative_mul_negative() -> None:
    a: int = 0 - 3
    b: int = 0 - 4
    result: int = a * b
    print(result)
    assert result == 12


def run_tests() -> None:
    test_precedence_mul_before_add()
    test_precedence_parentheses()
    test_precedence_nested_parens()
    test_precedence_sub_mul()
    test_precedence_mod()
    test_complex_expression()
    test_multi_term()
    test_subtraction_chain()
    test_mixed_operations()
    test_square()
    test_cube()
    test_mod_pattern()
    test_mod_zero_result()
    test_arithmetic_with_negative()
    test_double_negative()
    test_negative_mul()
    test_negative_mul_negative()
