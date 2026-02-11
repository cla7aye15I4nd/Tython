def fadd(a: float, b: float) -> float:
    return a + b


def fsub(a: float, b: float) -> float:
    return a - b


def fmul(a: float, b: float) -> float:
    return a * b


def fdiv(a: float, b: float) -> float:
    return a / b


def fmod(a: float, b: float) -> float:
    return a % b


def fabs(x: float) -> float:
    if x < 0.0:
        return 0.0 - x
    else:
        return x


def lerp(a: float, b: float, t: float) -> float:
    return fadd(fmul(a, fsub(1.0, t)), fmul(b, t))


def average3(a: float, b: float, c: float) -> float:
    return fdiv(fadd(fadd(a, b), c), 3.0)


def clamp_float(val: float, lo: float, hi: float) -> float:
    if val < lo:
        return lo
    elif val > hi:
        return hi
    else:
        return val


def distance_1d(a: float, b: float) -> float:
    return fabs(fsub(a, b))


def sum_float_range(n: int) -> float:
    total: float = 0.0
    i: int = 1
    while i <= n:
        total = fadd(total, float(i))
        i = i + 1
    return total


def newton_sqrt_steps(x: float, steps: int) -> float:
    guess: float = fdiv(x, 2.0)
    i: int = 0
    while i < steps:
        guess = fdiv(fadd(guess, fdiv(x, guess)), 2.0)
        i = i + 1
    return guess


# ── tests ─────────────────────────────────────────────────────────


def test_fadd_basic() -> None:
    result: float = fadd(1.5, 2.5)
    x: int = int(result)
    print(x)
    assert x == 4


def test_nested_float_ops() -> None:
    result: float = fmul(fadd(1.5, 2.5), fsub(10.0, 6.0))
    x: int = int(result)
    print(x)
    assert x == 16


def test_fdiv_chain() -> None:
    a: float = fdiv(100.0, 4.0)
    b: float = fdiv(a, 5.0)
    result: int = int(b)
    print(result)
    assert result == 5


def test_fmod_basic() -> None:
    result: float = fmod(10.5, 3.0)
    x: int = int(result)
    print(x)
    assert x == 1


def test_fabs_negative() -> None:
    result: float = fabs(0.0 - 7.5)
    x: int = int(result)
    print(x)
    assert x == 7


def test_fabs_positive() -> None:
    result: float = fabs(3.25)
    x: int = int(result)
    print(x)
    assert x == 3


def test_lerp_zero() -> None:
    result: float = lerp(10.0, 20.0, 0.0)
    x: int = int(result)
    print(x)
    assert x == 10


def test_lerp_one() -> None:
    result: float = lerp(10.0, 20.0, 1.0)
    x: int = int(result)
    print(x)
    assert x == 20


def test_lerp_half() -> None:
    result: float = lerp(0.0, 100.0, 0.5)
    x: int = int(result)
    print(x)
    assert x == 50


def test_lerp_composed() -> None:
    lo: float = fadd(5.0, 5.0)
    hi: float = fmul(lo, 3.0)
    result: float = lerp(lo, hi, 0.5)
    x: int = int(result)
    print(x)
    assert x == 20


def test_average3_uniform() -> None:
    result: float = average3(6.0, 6.0, 6.0)
    x: int = int(result)
    print(x)
    assert x == 6


def test_average3_varied() -> None:
    result: float = average3(3.0, 6.0, 9.0)
    x: int = int(result)
    print(x)
    assert x == 6


def test_average3_from_ops() -> None:
    a: float = fmul(2.0, 3.0)
    b: float = fdiv(24.0, 4.0)
    c: float = fadd(1.0, 2.0)
    result: float = average3(a, b, c)
    x: int = int(result)
    print(x)
    assert x == 5


def test_clamp_float_below() -> None:
    result: float = clamp_float(0.0 - 5.0, 0.0, 100.0)
    x: int = int(result)
    print(x)
    assert x == 0


def test_clamp_float_above() -> None:
    result: float = clamp_float(200.0, 0.0, 100.0)
    x: int = int(result)
    print(x)
    assert x == 100


def test_clamp_float_in_range() -> None:
    result: float = clamp_float(42.0, 0.0, 100.0)
    x: int = int(result)
    print(x)
    assert x == 42


def test_distance_1d_basic() -> None:
    result: float = distance_1d(3.0, 7.0)
    x: int = int(result)
    print(x)
    assert x == 4


def test_distance_1d_negative() -> None:
    result: float = distance_1d(7.0, 3.0)
    x: int = int(result)
    print(x)
    assert x == 4


def test_sum_float_range_basic() -> None:
    result: float = sum_float_range(10)
    x: int = int(result)
    print(x)
    assert x == 55


def test_sum_float_range_as_arg() -> None:
    s: float = sum_float_range(5)
    result: float = fmul(s, 2.0)
    x: int = int(result)
    print(x)
    assert x == 30


def test_newton_sqrt_of_4() -> None:
    result: float = newton_sqrt_steps(4.0, 10)
    x: int = int(result)
    print(x)
    assert x == 2


def test_newton_sqrt_of_9() -> None:
    result: float = newton_sqrt_steps(9.0, 10)
    x: int = int(result)
    print(x)
    assert x == 3


def test_complex_float_pipeline() -> None:
    a: float = sum_float_range(4)
    b: float = fdiv(a, 2.0)
    c: float = lerp(b, fmul(b, 3.0), 0.5)
    result: int = int(c)
    print(result)
    assert result == 10


def test_int_float_interop() -> None:
    n: int = 7
    x: float = float(n)
    y: float = fmul(x, 3.0)
    result: int = int(fsub(y, 1.0))
    print(result)
    assert result == 20


def run_tests() -> None:
    test_fadd_basic()
    test_nested_float_ops()
    test_fdiv_chain()
    test_fmod_basic()
    test_fabs_negative()
    test_fabs_positive()
    test_lerp_zero()
    test_lerp_one()
    test_lerp_half()
    test_lerp_composed()
    test_average3_uniform()
    test_average3_varied()
    test_average3_from_ops()
    test_clamp_float_below()
    test_clamp_float_above()
    test_clamp_float_in_range()
    test_distance_1d_basic()
    test_distance_1d_negative()
    test_sum_float_range_basic()
    test_sum_float_range_as_arg()
    test_newton_sqrt_of_4()
    test_newton_sqrt_of_9()
    test_complex_float_pipeline()
    test_int_float_interop()
