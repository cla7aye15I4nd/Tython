def test_if_true() -> int:
    result: int = 0
    if 1 == 1:
        result = 10
    print(result)
    assert result == 10
    return result


def test_if_false() -> int:
    result: int = 5
    if 1 == 2:
        result = 10
    print(result)
    assert result == 5
    return result


def test_if_else_true_branch() -> int:
    result: int = 0
    if 3 > 1:
        result = 100
    else:
        result = 200
    print(result)
    assert result == 100
    return result


def test_if_else_false_branch() -> int:
    result: int = 0
    if 1 > 3:
        result = 100
    else:
        result = 200
    print(result)
    assert result == 200
    return result


def test_if_elif_else_first() -> int:
    x: int = 1
    result: int = 0
    if x == 1:
        result = 10
    elif x == 2:
        result = 20
    else:
        result = 30
    print(result)
    assert result == 10
    return result


def test_if_elif_else_second() -> int:
    x: int = 2
    result: int = 0
    if x == 1:
        result = 10
    elif x == 2:
        result = 20
    else:
        result = 30
    print(result)
    assert result == 20
    return result


def test_if_elif_else_default() -> int:
    x: int = 99
    result: int = 0
    if x == 1:
        result = 10
    elif x == 2:
        result = 20
    else:
        result = 30
    print(result)
    assert result == 30
    return result


def test_if_multiple_elif() -> int:
    x: int = 4
    result: int = 0
    if x == 1:
        result = 10
    elif x == 2:
        result = 20
    elif x == 3:
        result = 30
    elif x == 4:
        result = 40
    else:
        result = 50
    print(result)
    assert result == 40
    return result


def test_nested_if() -> int:
    x: int = 10
    y: int = 20
    result: int = 0
    if x == 10:
        if y == 20:
            result = 1
        else:
            result = 2
    else:
        result = 3
    print(result)
    assert result == 1
    return result


def test_nested_if_outer_false() -> int:
    x: int = 5
    y: int = 20
    result: int = 0
    if x == 10:
        if y == 20:
            result = 1
        else:
            result = 2
    else:
        result = 3
    print(result)
    assert result == 3
    return result


def test_if_with_computation() -> int:
    a: int = 5
    b: int = 3
    result: int = 0
    if a + b == 8:
        result = a * b
    else:
        result = a - b
    print(result)
    assert result == 15
    return result


def test_if_gt() -> int:
    result: int = 0
    if 10 > 5:
        result = 1
    print(result)
    assert result == 1
    return result


def test_if_lt() -> int:
    result: int = 0
    if 3 < 8:
        result = 1
    print(result)
    assert result == 1
    return result


def test_if_gte() -> int:
    result: int = 0
    if 5 >= 5:
        result = 1
    print(result)
    assert result == 1
    return result


def test_if_lte() -> int:
    result: int = 0
    if 5 <= 5:
        result = 1
    print(result)
    assert result == 1
    return result


def test_if_neq() -> int:
    result: int = 0
    if 5 != 3:
        result = 1
    print(result)
    assert result == 1
    return result


def run_tests() -> None:
    test_if_true()
    test_if_false()
    test_if_else_true_branch()
    test_if_else_false_branch()
    test_if_elif_else_first()
    test_if_elif_else_second()
    test_if_elif_else_default()
    test_if_multiple_elif()
    test_nested_if()
    test_nested_if_outer_false()
    test_if_with_computation()
    test_if_gt()
    test_if_lt()
    test_if_gte()
    test_if_lte()
    test_if_neq()
