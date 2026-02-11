def is_positive(n: int) -> bool:
    return n > 0


def is_even(n: int) -> bool:
    return n % 2 == 0


def is_in_range(val: int, lo: int, hi: int) -> bool:
    return val >= lo and val <= hi


def both_true(a: bool, b: bool) -> bool:
    if a and b:
        return True
    else:
        return False


def either_true(a: bool, b: bool) -> bool:
    if a or b:
        return True
    else:
        return False


def negate(x: bool) -> bool:
    if x:
        return False
    else:
        return True


def bool_to_score(x: bool) -> int:
    if x:
        return 1
    else:
        return 0


def count_true3(a: bool, b: bool, c: bool) -> int:
    return bool_to_score(a) + bool_to_score(b) + bool_to_score(c)


def majority(a: bool, b: bool, c: bool) -> bool:
    return count_true3(a, b, c) >= 2


def xor(a: bool, b: bool) -> bool:
    return both_true(either_true(a, b), negate(both_true(a, b)))


def implies(a: bool, b: bool) -> bool:
    return either_true(negate(a), b)


def classify_sign(n: int) -> int:
    if is_positive(n):
        return 1
    elif n == 0:
        return 0
    else:
        return 0 - 1


# ── tests ─────────────────────────────────────────────────────────


def test_is_positive_true() -> None:
    result: bool = is_positive(42)
    x: int = int(result)
    print(x)
    assert x == 1


def test_is_positive_false() -> None:
    result: bool = is_positive(0 - 3)
    x: int = int(result)
    print(x)
    assert x == 0


def test_is_positive_zero() -> None:
    result: bool = is_positive(0)
    x: int = int(result)
    print(x)
    assert x == 0


def test_is_even_true() -> None:
    result: bool = is_even(10)
    x: int = int(result)
    print(x)
    assert x == 1


def test_is_even_false() -> None:
    result: bool = is_even(7)
    x: int = int(result)
    print(x)
    assert x == 0


def test_is_in_range_inside() -> None:
    result: bool = is_in_range(50, 0, 100)
    x: int = int(result)
    print(x)
    assert x == 1


def test_is_in_range_below() -> None:
    result: bool = is_in_range(0 - 1, 0, 100)
    x: int = int(result)
    print(x)
    assert x == 0


def test_is_in_range_boundary() -> None:
    lo: bool = is_in_range(0, 0, 100)
    hi: bool = is_in_range(100, 0, 100)
    result: bool = both_true(lo, hi)
    x: int = int(result)
    print(x)
    assert x == 1


def test_both_true_tt() -> None:
    result: bool = both_true(True, True)
    x: int = int(result)
    print(x)
    assert x == 1


def test_both_true_tf() -> None:
    result: bool = both_true(True, False)
    x: int = int(result)
    print(x)
    assert x == 0


def test_either_true_ff() -> None:
    result: bool = either_true(False, False)
    x: int = int(result)
    print(x)
    assert x == 0


def test_either_true_tf() -> None:
    result: bool = either_true(True, False)
    x: int = int(result)
    print(x)
    assert x == 1


def test_negate_true() -> None:
    result: bool = negate(True)
    x: int = int(result)
    print(x)
    assert x == 0


def test_negate_false() -> None:
    result: bool = negate(False)
    x: int = int(result)
    print(x)
    assert x == 1


def test_negate_from_comparison() -> None:
    cmp: bool = 5 > 10
    result: bool = negate(cmp)
    x: int = int(result)
    print(x)
    assert x == 1


def test_count_true3_all() -> None:
    result: int = count_true3(True, True, True)
    print(result)
    assert result == 3


def test_count_true3_none() -> None:
    result: int = count_true3(False, False, False)
    print(result)
    assert result == 0


def test_count_true3_mixed() -> None:
    a: bool = is_positive(5)
    b: bool = is_even(7)
    c: bool = is_in_range(50, 0, 100)
    result: int = count_true3(a, b, c)
    print(result)
    assert result == 2


def test_majority_true() -> None:
    result: bool = majority(True, True, False)
    x: int = int(result)
    print(x)
    assert x == 1


def test_majority_false() -> None:
    result: bool = majority(True, False, False)
    x: int = int(result)
    print(x)
    assert x == 0


def test_majority_from_checks() -> None:
    a: bool = is_positive(10)
    b: bool = is_even(10)
    c: bool = is_in_range(10, 20, 30)
    result: bool = majority(a, b, c)
    x: int = int(result)
    print(x)
    assert x == 1


def test_xor_different() -> None:
    result: bool = xor(True, False)
    x: int = int(result)
    print(x)
    assert x == 1


def test_xor_same() -> None:
    result: bool = xor(True, True)
    x: int = int(result)
    print(x)
    assert x == 0


def test_xor_composed() -> None:
    a: bool = is_positive(5)
    b: bool = is_positive(0 - 3)
    result: bool = xor(a, b)
    x: int = int(result)
    print(x)
    assert x == 1


def test_implies_true_true() -> None:
    result: bool = implies(True, True)
    x: int = int(result)
    print(x)
    assert x == 1


def test_implies_true_false() -> None:
    result: bool = implies(True, False)
    x: int = int(result)
    print(x)
    assert x == 0


def test_implies_false_any() -> None:
    r1: bool = implies(False, True)
    r2: bool = implies(False, False)
    result: bool = both_true(r1, r2)
    x: int = int(result)
    print(x)
    assert x == 1


def test_classify_positive() -> None:
    result: int = classify_sign(42)
    print(result)
    assert result == 1


def test_classify_negative() -> None:
    result: int = classify_sign(0 - 7)
    print(result)
    assert result == 0 - 1


def test_classify_zero() -> None:
    result: int = classify_sign(0)
    print(result)
    assert result == 0


def test_bool_chain_logic() -> None:
    a: bool = is_positive(10)
    b: bool = is_even(10)
    c: bool = both_true(a, b)
    d: bool = negate(is_in_range(10, 20, 30))
    result: bool = both_true(c, d)
    x: int = int(result)
    print(x)
    assert x == 1


def test_bool_accumulate_loop() -> None:
    count: int = 0
    i: int = 1
    while i <= 10:
        if is_even(i):
            count = count + bool_to_score(is_in_range(i, 4, 8))
        i = i + 1
    print(count)
    assert count == 3


def run_tests() -> None:
    test_is_positive_true()
    test_is_positive_false()
    test_is_positive_zero()
    test_is_even_true()
    test_is_even_false()
    test_is_in_range_inside()
    test_is_in_range_below()
    test_is_in_range_boundary()
    test_both_true_tt()
    test_both_true_tf()
    test_either_true_ff()
    test_either_true_tf()
    test_negate_true()
    test_negate_false()
    test_negate_from_comparison()
    test_count_true3_all()
    test_count_true3_none()
    test_count_true3_mixed()
    test_majority_true()
    test_majority_false()
    test_majority_from_checks()
    test_xor_different()
    test_xor_same()
    test_xor_composed()
    test_implies_true_true()
    test_implies_true_false()
    test_implies_false_any()
    test_classify_positive()
    test_classify_negative()
    test_classify_zero()
    test_bool_chain_logic()
    test_bool_accumulate_loop()
