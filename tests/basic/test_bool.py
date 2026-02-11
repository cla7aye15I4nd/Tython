def is_positive(n: int) -> bool:
    return n > 0


def is_negative(n: int) -> bool:
    return n < 0


def is_zero(n: int) -> bool:
    return n == 0


def is_even(n: int) -> bool:
    return n % 2 == 0


def is_odd(n: int) -> bool:
    return negate(is_even(n))


def is_divisible(n: int, d: int) -> bool:
    return n % d == 0


def is_in_range(val: int, lo: int, hi: int) -> bool:
    return both(val >= lo, val <= hi)


def negate(x: bool) -> bool:
    if x:
        return False
    else:
        return True


def both(a: bool, b: bool) -> bool:
    if a:
        return b
    else:
        return False


def either(a: bool, b: bool) -> bool:
    if a:
        return True
    else:
        return b


def xor(a: bool, b: bool) -> bool:
    return both(either(a, b), negate(both(a, b)))


def implies(a: bool, b: bool) -> bool:
    return either(negate(a), b)


def iff(a: bool, b: bool) -> bool:
    return both(implies(a, b), implies(b, a))


def bool_to_int(x: bool) -> int:
    if x:
        return 1
    else:
        return 0


def count_true3(a: bool, b: bool, c: bool) -> int:
    return bool_to_int(a) + bool_to_int(b) + bool_to_int(c)


def count_true4(a: bool, b: bool, c: bool, d: bool) -> int:
    return count_true3(a, b, c) + bool_to_int(d)


def majority3(a: bool, b: bool, c: bool) -> bool:
    return count_true3(a, b, c) >= 2


def unanimous3(a: bool, b: bool, c: bool) -> bool:
    return count_true3(a, b, c) == 3


def any_of4(a: bool, b: bool, c: bool, d: bool) -> bool:
    return count_true4(a, b, c, d) >= 1


def none_of4(a: bool, b: bool, c: bool, d: bool) -> bool:
    return negate(any_of4(a, b, c, d))


def exactly_n(n: int, a: bool, b: bool, c: bool, d: bool) -> bool:
    return count_true4(a, b, c, d) == n


def classify_sign(n: int) -> int:
    if is_positive(n):
        return 1
    elif is_negative(n):
        return 0 - 1
    else:
        return 0


def abs_val(n: int) -> int:
    if is_negative(n):
        return 0 - n
    else:
        return n


def safe_div_positive(a: int, b: int) -> int:
    if both(is_positive(b), negate(is_zero(b))):
        return a // b
    else:
        return 0


def is_prime(n: int) -> bool:
    if n < 2:
        return False
    i: int = 2
    while i * i <= n:
        if is_divisible(n, i):
            return False
        i = i + 1
    return True


def is_perfect_square(n: int) -> bool:
    if is_negative(n):
        return False
    i: int = 0
    while i * i <= n:
        if i * i == n:
            return True
        i = i + 1
    return False


def all_positive_range(lo: int, hi: int) -> bool:
    i: int = lo
    while i <= hi:
        if negate(is_positive(i)):
            return False
        i = i + 1
    return True


def has_prime_in_range(lo: int, hi: int) -> bool:
    i: int = lo
    while i <= hi:
        if is_prime(i):
            return True
        i = i + 1
    return False


def count_primes_in_range(lo: int, hi: int) -> int:
    count: int = 0
    i: int = lo
    while i <= hi:
        count = count + bool_to_int(is_prime(i))
        i = i + 1
    return count


def fizzbuzz_class(n: int) -> int:
    d3: bool = is_divisible(n, 3)
    d5: bool = is_divisible(n, 5)
    if both(d3, d5):
        return 3
    elif d3:
        return 1
    elif d5:
        return 2
    else:
        return 0


def grade(score: int) -> int:
    if is_in_range(score, 90, 100):
        return 4
    elif is_in_range(score, 80, 89):
        return 3
    elif is_in_range(score, 70, 79):
        return 2
    elif is_in_range(score, 60, 69):
        return 1
    else:
        return 0


def multi_condition_check(a: int, b: int, c: int, threshold: int) -> bool:
    pos_a: bool = is_positive(a)
    pos_b: bool = is_positive(b)
    pos_c: bool = is_positive(c)
    all_pos: bool = unanimous3(pos_a, pos_b, pos_c)
    sum_ok: bool = a + b + c > threshold
    return both(all_pos, sum_ok)


# ── tests ─────────────────────────────────────────────────────────


def test_is_odd_via_negate() -> None:
    a: bool = is_odd(7)
    b: bool = is_odd(10)
    result: int = bool_to_int(a) + bool_to_int(b)
    print(result)
    assert result == 1


def test_is_divisible_chain() -> None:
    d2: bool = is_divisible(30, 2)
    d3: bool = is_divisible(30, 3)
    d5: bool = is_divisible(30, 5)
    d7: bool = is_divisible(30, 7)
    result: int = count_true4(d2, d3, d5, d7)
    print(result)
    assert result == 3


def test_is_in_range_composed() -> None:
    val: int = 7 * 8
    lo: int = 50
    hi: int = 60
    result: int = bool_to_int(is_in_range(val, lo, hi))
    print(result)
    assert result == 1


def test_xor_truth_table() -> None:
    r1: int = bool_to_int(xor(True, False))
    r2: int = bool_to_int(xor(False, True))
    r3: int = bool_to_int(xor(True, True))
    r4: int = bool_to_int(xor(False, False))
    result: int = r1 + r2 + r3 + r4
    print(result)
    assert result == 2


def test_implies_truth_table() -> None:
    r1: int = bool_to_int(implies(True, True))
    r2: int = bool_to_int(implies(True, False))
    r3: int = bool_to_int(implies(False, True))
    r4: int = bool_to_int(implies(False, False))
    result: int = r1 + r2 + r3 + r4
    print(result)
    assert result == 3


def test_iff_same() -> None:
    a: bool = iff(True, True)
    b: bool = iff(False, False)
    result: int = bool_to_int(both(a, b))
    print(result)
    assert result == 1


def test_iff_different() -> None:
    a: bool = iff(True, False)
    b: bool = iff(False, True)
    result: int = bool_to_int(either(a, b))
    print(result)
    assert result == 0


def test_iff_from_predicates() -> None:
    a: bool = is_even(14)
    b: bool = is_divisible(14, 2)
    result: int = bool_to_int(iff(a, b))
    print(result)
    assert result == 1


def test_count_true4_from_predicates() -> None:
    a: bool = is_positive(10)
    b: bool = is_even(10)
    c: bool = is_prime(10)
    d: bool = is_in_range(10, 5, 15)
    result: int = count_true4(a, b, c, d)
    print(result)
    assert result == 3


def test_majority3_from_complex() -> None:
    a: bool = is_prime(17)
    b: bool = is_even(17)
    c: bool = is_in_range(17, 10, 20)
    result: int = bool_to_int(majority3(a, b, c))
    print(result)
    assert result == 1


def test_unanimous3_all_true() -> None:
    a: bool = is_positive(5)
    b: bool = is_odd(5)
    c: bool = is_prime(5)
    result: int = bool_to_int(unanimous3(a, b, c))
    print(result)
    assert result == 1


def test_unanimous3_one_false() -> None:
    a: bool = is_positive(4)
    b: bool = is_even(4)
    c: bool = is_prime(4)
    result: int = bool_to_int(unanimous3(a, b, c))
    print(result)
    assert result == 0


def test_any_of4_one_true() -> None:
    a: bool = is_prime(4)
    b: bool = is_negative(5)
    c: bool = is_zero(0)
    d: bool = is_even(7)
    result: int = bool_to_int(any_of4(a, b, c, d))
    print(result)
    assert result == 1


def test_none_of4_all_false() -> None:
    a: bool = is_prime(4)
    b: bool = is_negative(5)
    c: bool = is_even(7)
    d: bool = is_zero(3)
    result: int = bool_to_int(none_of4(a, b, c, d))
    print(result)
    assert result == 1


def test_exactly_n_two() -> None:
    a: bool = is_prime(7)
    b: bool = is_even(7)
    c: bool = is_positive(7)
    d: bool = is_divisible(7, 3)
    result: int = bool_to_int(exactly_n(2, a, b, c, d))
    print(result)
    assert result == 1


def test_classify_sign_chain() -> None:
    s1: int = classify_sign(42)
    s2: int = classify_sign(0 - 7)
    s3: int = classify_sign(0)
    result: int = s1 + s2 + s3
    print(result)
    assert result == 0


def test_abs_val_uses_is_negative() -> None:
    a: int = abs_val(0 - 25)
    b: int = abs_val(25)
    result: int = bool_to_int(a == b)
    print(result)
    assert result == 1


def test_safe_div_guards() -> None:
    a: int = safe_div_positive(100, 7)
    b: int = safe_div_positive(100, 0)
    c: int = safe_div_positive(100, 0 - 5)
    result: int = a + b + c
    print(result)
    assert result == 14


def test_is_prime_loop() -> None:
    total: int = 0
    i: int = 2
    while i <= 20:
        total = total + bool_to_int(is_prime(i))
        i = i + 1
    print(total)
    assert total == 8


def test_is_perfect_square_loop() -> None:
    count: int = 0
    i: int = 0
    while i <= 25:
        count = count + bool_to_int(is_perfect_square(i))
        i = i + 1
    print(count)
    assert count == 6


def test_all_positive_range_true() -> None:
    result: int = bool_to_int(all_positive_range(1, 10))
    print(result)
    assert result == 1


def test_all_positive_range_false() -> None:
    result: int = bool_to_int(all_positive_range(0 - 2, 5))
    print(result)
    assert result == 0


def test_has_prime_in_range_true() -> None:
    result: int = bool_to_int(has_prime_in_range(20, 30))
    print(result)
    assert result == 1


def test_has_prime_in_range_false() -> None:
    result: int = bool_to_int(has_prime_in_range(24, 28))
    print(result)
    assert result == 0


def test_count_primes_in_range_basic() -> None:
    result: int = count_primes_in_range(1, 30)
    print(result)
    assert result == 10


def test_count_primes_feeds_is_even() -> None:
    c: int = count_primes_in_range(1, 50)
    result: int = bool_to_int(is_odd(c))
    print(result)
    assert result == 1


def test_fizzbuzz_class_all_cases() -> None:
    fb15: int = fizzbuzz_class(15)
    fb9: int = fizzbuzz_class(9)
    fb10: int = fizzbuzz_class(10)
    fb7: int = fizzbuzz_class(7)
    result: int = fb15 * 1000 + fb9 * 100 + fb10 * 10 + fb7
    print(result)
    assert result == 3120


def test_fizzbuzz_loop_count() -> None:
    fizz_count: int = 0
    buzz_count: int = 0
    fizzbuzz_count: int = 0
    i: int = 1
    while i <= 30:
        cls: int = fizzbuzz_class(i)
        if cls == 1:
            fizz_count = fizz_count + 1
        elif cls == 2:
            buzz_count = buzz_count + 1
        elif cls == 3:
            fizzbuzz_count = fizzbuzz_count + 1
        i = i + 1
    result: int = fizz_count * 100 + buzz_count * 10 + fizzbuzz_count
    print(result)
    assert result == 842


def test_grade_all_levels() -> None:
    g95: int = grade(95)
    g85: int = grade(85)
    g75: int = grade(75)
    g65: int = grade(65)
    g50: int = grade(50)
    result: int = g95 + g85 + g75 + g65 + g50
    print(result)
    assert result == 10


def test_grade_from_computed_score() -> None:
    base: int = 15
    score: int = base * 5 + bool_to_int(is_prime(base)) * 10
    result: int = grade(score)
    print(result)
    assert result == 2


def test_multi_condition_check_pass() -> None:
    result: int = bool_to_int(multi_condition_check(10, 20, 30, 50))
    print(result)
    assert result == 1


def test_multi_condition_check_fail_sum() -> None:
    result: int = bool_to_int(multi_condition_check(5, 3, 2, 50))
    print(result)
    assert result == 0


def test_multi_condition_check_fail_negative() -> None:
    result: int = bool_to_int(multi_condition_check(100, 0 - 1, 50, 10))
    print(result)
    assert result == 0


def test_complex_predicate_pipeline() -> None:
    n: int = 42
    a: bool = is_even(n)
    b: bool = is_divisible(n, 7)
    c: bool = is_in_range(n, 40, 50)
    d: bool = negate(is_prime(n))
    all4: bool = exactly_n(4, a, b, c, d)
    g: int = grade(n + 50)
    result: int = bool_to_int(all4) * 10 + g
    print(result)
    assert result == 14


def test_nested_bool_decision_tree() -> None:
    n: int = 30
    step1: bool = is_divisible(n, 2)
    step2: bool = is_divisible(n, 3)
    step3: bool = is_divisible(n, 5)
    branch1: bool = unanimous3(step1, step2, step3)
    result: int = 0
    if branch1:
        sub_check: bool = is_perfect_square(n)
        if sub_check:
            result = 100
        else:
            count = count_primes_in_range(1, n)
            result = count
    else:
        result = 0 - 1
    print(result)
    assert result == 10


def test_loop_with_multiple_predicates() -> None:
    score: int = 0
    i: int = 1
    while i <= 20:
        p: bool = is_prime(i)
        e: bool = is_even(i)
        sq: bool = is_perfect_square(i)
        if both(p, is_odd(i)):
            score = score + 3
        elif both(e, sq):
            score = score + 5
        elif either(p, sq):
            score = score + 2
        else:
            score = score + 1
        i = i + 1
    print(score)
    assert score == 45


def test_double_accumulation_bools() -> None:
    true_count: int = 0
    false_count: int = 0
    i: int = 1
    while i <= 15:
        check: bool = both(is_odd(i), is_in_range(i, 5, 15))
        true_count = true_count + bool_to_int(check)
        false_count = false_count + bool_to_int(negate(check))
        i = i + 1
    result: int = true_count * 100 + false_count
    print(result)
    assert result == 609


def run_tests() -> None:
    test_is_odd_via_negate()
    test_is_divisible_chain()
    test_is_in_range_composed()
    test_xor_truth_table()
    test_implies_truth_table()
    test_iff_same()
    test_iff_different()
    test_iff_from_predicates()
    test_count_true4_from_predicates()
    test_majority3_from_complex()
    test_unanimous3_all_true()
    test_unanimous3_one_false()
    test_any_of4_one_true()
    test_none_of4_all_false()
    test_exactly_n_two()
    test_classify_sign_chain()
    test_abs_val_uses_is_negative()
    test_safe_div_guards()
    test_is_prime_loop()
    test_is_perfect_square_loop()
    test_all_positive_range_true()
    test_all_positive_range_false()
    test_has_prime_in_range_true()
    test_has_prime_in_range_false()
    test_count_primes_in_range_basic()
    test_count_primes_feeds_is_even()
    test_fizzbuzz_class_all_cases()
    test_fizzbuzz_loop_count()
    test_grade_all_levels()
    test_grade_from_computed_score()
    test_multi_condition_check_pass()
    test_multi_condition_check_fail_sum()
    test_multi_condition_check_fail_negative()
    test_complex_predicate_pipeline()
    test_nested_bool_decision_tree()
    test_loop_with_multiple_predicates()
    test_double_accumulation_bools()
