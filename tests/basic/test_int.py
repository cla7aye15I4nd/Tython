def square(n: int) -> int:
    return n * n


def add(a: int, b: int) -> int:
    return a + b


def sub(a: int, b: int) -> int:
    return a - b


def mul(a: int, b: int) -> int:
    return a * b


def mod(a: int, b: int) -> int:
    return a % b


def sum_range(lo: int, hi: int) -> int:
    total: int = 0
    i: int = lo
    while i <= hi:
        total = total + i
        i = i + 1
    return total


def factorial(n: int) -> int:
    result: int = 1
    i: int = 2
    while i <= n:
        result = result * i
        i = i + 1
    return result


def clamp(val: int, lo: int, hi: int) -> int:
    if val < lo:
        return lo
    elif val > hi:
        return hi
    else:
        return val


def abs_diff(a: int, b: int) -> int:
    if a > b:
        return a - b
    else:
        return b - a


def combine(a: int, b: int, c: int) -> int:
    return add(mul(a, b), c)


def gcd(a: int, b: int) -> int:
    while b != 0:
        temp: int = b
        b = a % b
        a = temp
    return a


def power(base: int, exp: int) -> int:
    result: int = 1
    i: int = 0
    while i < exp:
        result = mul(result, base)
        i = i + 1
    return result


# ── tests ─────────────────────────────────────────────────────────


def test_square_and_add() -> None:
    a: int = square(7)
    b: int = square(3)
    result: int = add(a, b)
    print(result)
    assert result == 58


def test_nested_arithmetic() -> None:
    x: int = mul(add(3, 4), sub(10, 5))
    print(x)
    assert x == 35


def test_combine_three_args() -> None:
    result: int = combine(5, 6, 7)
    print(result)
    assert result == 37


def test_chain_operations() -> None:
    a: int = add(10, 20)
    b: int = mul(a, 3)
    c: int = sub(b, square(5))
    d: int = mod(c, 11)
    print(d)
    assert d == 10


def test_sum_range_basic() -> None:
    result: int = sum_range(1, 10)
    print(result)
    assert result == 55


def test_sum_range_as_arg() -> None:
    s1: int = sum_range(1, 5)
    s2: int = sum_range(6, 10)
    result: int = add(s1, s2)
    print(result)
    assert result == 55


def test_factorial_values() -> None:
    f5: int = factorial(5)
    f3: int = factorial(3)
    result: int = sub(f5, f3)
    print(result)
    assert result == 114


def test_clamp_below() -> None:
    result: int = clamp(0 - 50, 0, 100)
    print(result)
    assert result == 0


def test_clamp_above() -> None:
    result: int = clamp(200, 0, 100)
    print(result)
    assert result == 100


def test_clamp_in_range() -> None:
    result: int = clamp(42, 0, 100)
    print(result)
    assert result == 42


def test_clamp_chain() -> None:
    raw: int = mul(25, 8)
    clamped: int = clamp(raw, 10, 150)
    result: int = add(clamped, square(3))
    print(result)
    assert result == 159


def test_abs_diff_ordered() -> None:
    result: int = abs_diff(20, 7)
    print(result)
    assert result == 13


def test_abs_diff_reversed() -> None:
    result: int = abs_diff(7, 20)
    print(result)
    assert result == 13


def test_abs_diff_of_squares() -> None:
    result: int = abs_diff(square(5), square(4))
    print(result)
    assert result == 9


def test_gcd_basic() -> None:
    result: int = gcd(48, 18)
    print(result)
    assert result == 6


def test_gcd_coprime() -> None:
    result: int = gcd(17, 13)
    print(result)
    assert result == 1


def test_gcd_composed() -> None:
    g1: int = gcd(100, 75)
    g2: int = gcd(60, 45)
    result: int = mul(g1, g2)
    print(result)
    assert result == 375


def test_power_basic() -> None:
    result: int = power(2, 10)
    print(result)
    assert result == 1024


def test_power_composed() -> None:
    base: int = add(1, 2)
    exp: int = sub(7, 3)
    result: int = power(base, exp)
    print(result)
    assert result == 81


def test_complex_pipeline() -> None:
    a: int = sum_range(1, 4)
    b: int = factorial(a)
    c: int = mod(b, power(2, 5))
    d: int = add(c, clamp(999, 0, 50))
    result: int = square(gcd(d, 60))
    print(result)
    assert result == 100


def test_multi_step_accumulate() -> None:
    acc: int = 0
    i: int = 1
    while i <= 5:
        acc = add(acc, mul(i, i))
        i = i + 1
    result: int = acc
    print(result)
    assert result == 55


def test_iterative_gcd_chain() -> None:
    result: int = gcd(gcd(120, 84), gcd(60, 45))
    print(result)
    assert result == 3


def run_tests() -> None:
    test_square_and_add()
    test_nested_arithmetic()
    test_combine_three_args()
    test_chain_operations()
    test_sum_range_basic()
    test_sum_range_as_arg()
    test_factorial_values()
    test_clamp_below()
    test_clamp_above()
    test_clamp_in_range()
    test_clamp_chain()
    test_abs_diff_ordered()
    test_abs_diff_reversed()
    test_abs_diff_of_squares()
    test_gcd_basic()
    test_gcd_coprime()
    test_gcd_composed()
    test_power_basic()
    test_power_composed()
    test_complex_pipeline()
    test_multi_step_accumulate()
    test_iterative_gcd_chain()
