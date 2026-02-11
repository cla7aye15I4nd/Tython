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
        return fsub(0.0, x)
    else:
        return x


def fmax(a: float, b: float) -> float:
    if a >= b:
        return a
    else:
        return b


def fmin(a: float, b: float) -> float:
    if a <= b:
        return a
    else:
        return b


def fclamp(val: float, lo: float, hi: float) -> float:
    return fmax(lo, fmin(val, hi))


def fsquare(x: float) -> float:
    return fmul(x, x)


def fcube(x: float) -> float:
    return fmul(fsquare(x), x)


def lerp(a: float, b: float, t: float) -> float:
    return fadd(fmul(a, fsub(1.0, t)), fmul(b, t))


def inverse_lerp(a: float, b: float, val: float) -> float:
    return fdiv(fsub(val, a), fsub(b, a))


def remap(val: float, in_lo: float, in_hi: float, out_lo: float, out_hi: float) -> float:
    t: float = inverse_lerp(in_lo, in_hi, val)
    return lerp(out_lo, out_hi, fclamp(t, 0.0, 1.0))


def weighted_avg4(a: float, wa: float, b: float, wb: float) -> float:
    return fdiv(fadd(fmul(a, wa), fmul(b, wb)), fadd(wa, wb))


def distance_2d(x1: float, y1: float, x2: float, y2: float) -> float:
    dx: float = fsub(x2, x1)
    dy: float = fsub(y2, y1)
    return newton_sqrt(fadd(fsquare(dx), fsquare(dy)), 15)


def sum_float_range(lo: int, hi: int, step: int) -> float:
    total: float = 0.0
    i: int = lo
    while i <= hi:
        total = fadd(total, float(i))
        i = i + step
    return total


def fpower(base: float, exp: int) -> float:
    result: float = 1.0
    i: int = 0
    while i < exp:
        result = fmul(result, base)
        i = i + 1
    return result


def newton_sqrt(x: float, steps: int) -> float:
    if x <= 0.0:
        return 0.0
    guess: float = fdiv(x, 2.0)
    i: int = 0
    while i < steps:
        guess = fdiv(fadd(guess, fdiv(x, guess)), 2.0)
        i = i + 1
    return guess


def geometric_series(a: float, r: float, n: int) -> float:
    total: float = 0.0
    term: float = a
    i: int = 0
    while i < n:
        total = fadd(total, term)
        term = fmul(term, r)
        i = i + 1
    return total


def taylor_exp_terms(x: float, terms: int) -> float:
    result: float = 1.0
    term: float = 1.0
    i: int = 1
    while i < terms:
        term = fdiv(fmul(term, x), float(i))
        result = fadd(result, term)
        i = i + 1
    return result


def bisect_sqrt(x: float, iterations: int) -> float:
    if x <= 0.0:
        return 0.0
    lo: float = 0.0
    hi: float = fmax(x, 1.0)
    i: int = 0
    while i < iterations:
        mid: float = fdiv(fadd(lo, hi), 2.0)
        if fmul(mid, mid) < x:
            lo = mid
        else:
            hi = mid
        i = i + 1
    return fdiv(fadd(lo, hi), 2.0)


def harmonic(n: int) -> float:
    total: float = 0.0
    i: int = 1
    while i <= n:
        total = fadd(total, fdiv(1.0, float(i)))
        i = i + 1
    return total


def moving_average_step(prev_avg: float, new_val: float, weight: float) -> float:
    return fadd(fmul(prev_avg, fsub(1.0, weight)), fmul(new_val, weight))


def smooth_step(edge0: float, edge1: float, x: float) -> float:
    t: float = fclamp(fdiv(fsub(x, edge0), fsub(edge1, edge0)), 0.0, 1.0)
    return fmul(fmul(t, t), fsub(3.0, fmul(2.0, t)))


# ── tests ─────────────────────────────────────────────────────────


def test_fsquare_of_sum() -> None:
    s: float = fadd(1.5, 2.5)
    result: int = int(fsquare(s))
    print(result)
    assert result == 16


def test_fcube_nested() -> None:
    base: float = fsub(5.0, 2.0)
    result: int = int(fcube(base))
    print(result)
    assert result == 27


def test_fabs_of_expression() -> None:
    x: float = fsub(3.0, 10.0)
    result: int = int(fabs(x))
    print(result)
    assert result == 7


def test_fclamp_via_fmax_fmin() -> None:
    raw: float = fmul(fadd(8.0, 7.0), 12.0)
    result: int = int(fclamp(raw, 50.0, 150.0))
    print(result)
    assert result == 150


def test_weighted_avg4_balanced() -> None:
    result: int = int(weighted_avg4(10.0, 1.0, 20.0, 1.0))
    print(result)
    assert result == 15


def test_weighted_avg4_from_ops() -> None:
    a: float = fsquare(4.0)
    b: float = fcube(2.0)
    result: int = int(weighted_avg4(a, 3.0, b, 1.0))
    print(result)
    assert result == 14


def test_lerp_quarter() -> None:
    lo: float = fmul(5.0, 2.0)
    hi: float = fmul(lo, 5.0)
    result: int = int(lerp(lo, hi, 0.25))
    print(result)
    assert result == 20


def test_inverse_lerp_roundtrip() -> None:
    t: float = 0.5
    lo: float = 10.0
    hi: float = 30.0
    val: float = lerp(lo, hi, t)
    recovered_t: float = inverse_lerp(lo, hi, val)
    result: int = int(fmul(recovered_t, 100.0))
    print(result)
    assert result == 50


def test_remap_basic() -> None:
    result: int = int(remap(50.0, 0.0, 100.0, 0.0, 10.0))
    print(result)
    assert result == 5


def test_remap_with_clamp() -> None:
    result: int = int(remap(200.0, 0.0, 100.0, 0.0, 50.0))
    print(result)
    assert result == 50


def test_remap_from_computed_bounds() -> None:
    in_lo: float = fsquare(2.0)
    in_hi: float = fsquare(10.0)
    val: float = fsquare(6.0)
    result: int = int(remap(val, in_lo, in_hi, 0.0, 100.0))
    print(result)
    assert result == 33


def test_distance_2d_basic() -> None:
    d: float = distance_2d(0.0, 0.0, 3.0, 4.0)
    result: int = int(d)
    print(result)
    assert result == 5


def test_distance_2d_from_ops() -> None:
    x1: float = fmul(2.0, 3.0)
    y1: float = fadd(1.0, 1.0)
    x2: float = fmul(2.0, 3.0)
    y2: float = fadd(1.0, 6.0)
    d: float = distance_2d(x1, y1, x2, y2)
    result: int = int(d)
    print(result)
    assert result == 5


def test_sum_float_range_with_step() -> None:
    s: float = sum_float_range(1, 20, 3)
    result: int = int(s)
    print(result)
    assert result == 70


def test_sum_range_feeds_sqrt() -> None:
    s: float = sum_float_range(1, 8, 1)
    result: int = int(newton_sqrt(s, 15))
    print(result)
    assert result == 6


def test_fpower_basic() -> None:
    result: int = int(fpower(2.0, 10))
    print(result)
    assert result == 1024


def test_fpower_nested_args() -> None:
    base: float = fadd(1.0, fsub(5.0, 3.0))
    result: int = int(fpower(base, 4))
    print(result)
    assert result == 81


def test_newton_sqrt_of_computed() -> None:
    val: float = fadd(fsquare(5.0), fsquare(12.0))
    result: int = int(newton_sqrt(val, 15))
    print(result)
    assert result == 13


def test_bisect_sqrt_of_16() -> None:
    result: int = int(bisect_sqrt(16.0, 50))
    print(result)
    assert result == 3


def test_bisect_sqrt_of_2_approx() -> None:
    approx: float = bisect_sqrt(2.0, 50)
    check: int = int(fmul(approx, approx))
    print(check)
    assert check == 1


def test_geometric_series_basic() -> None:
    result: int = int(geometric_series(1.0, 2.0, 10))
    print(result)
    assert result == 1023


def test_geometric_series_half() -> None:
    result: int = int(fmul(geometric_series(1.0, 0.5, 20), 100.0))
    print(result)
    assert result == 199


def test_taylor_exp_at_1() -> None:
    e_approx: float = taylor_exp_terms(1.0, 15)
    result: int = int(fmul(e_approx, 1000.0))
    print(result)
    assert result == 2718


def test_taylor_exp_at_2() -> None:
    e2: float = taylor_exp_terms(2.0, 20)
    result: int = int(fmul(e2, 100.0))
    print(result)
    assert result == 738


def test_harmonic_10() -> None:
    h: float = harmonic(10)
    result: int = int(fmul(h, 100.0))
    print(result)
    assert result == 292


def test_harmonic_feeds_lerp() -> None:
    h5: float = harmonic(5)
    h10: float = harmonic(10)
    result: int = int(fmul(lerp(h5, h10, 0.5), 100.0))
    print(result)
    assert result == 260


def test_moving_average_chain() -> None:
    avg: float = 0.0
    avg = moving_average_step(avg, 10.0, 0.5)
    avg = moving_average_step(avg, 20.0, 0.5)
    avg = moving_average_step(avg, 30.0, 0.5)
    avg = moving_average_step(avg, 40.0, 0.5)
    result: int = int(avg)
    print(result)
    assert result == 30


def test_moving_average_loop() -> None:
    avg: float = 0.0
    i: int = 1
    while i <= 10:
        avg = moving_average_step(avg, float(i), 0.3)
        i = i + 1
    result: int = int(fmul(avg, 10.0))
    print(result)
    assert result == 77


def test_smooth_step_edges() -> None:
    lo: int = int(fmul(smooth_step(0.0, 1.0, 0.0), 100.0))
    hi: int = int(fmul(smooth_step(0.0, 1.0, 1.0), 100.0))
    print(lo)
    print(hi)
    assert lo == 0
    assert hi == 100


def test_smooth_step_mid() -> None:
    result: int = int(fmul(smooth_step(0.0, 1.0, 0.5), 100.0))
    print(result)
    assert result == 50


def test_smooth_step_from_remap() -> None:
    t: float = remap(75.0, 0.0, 100.0, 0.0, 1.0)
    s: float = smooth_step(0.0, 1.0, t)
    result: int = int(fmul(s, 1000.0))
    print(result)
    assert result == 843


def test_mega_float_pipeline() -> None:
    base: float = newton_sqrt(sum_float_range(1, 10, 1), 15)
    geo: float = geometric_series(base, 0.5, 5)
    clamped: float = fclamp(geo, 0.0, 20.0)
    smoothed: float = smooth_step(0.0, 20.0, clamped)
    result: int = int(fmul(smoothed, 1000.0))
    print(result)
    assert result == 806


def test_convergent_float_branches() -> None:
    left: float = fpower(1.5, 6)
    right: float = harmonic(8)
    avg: float = weighted_avg4(left, 2.0, right, 3.0)
    d: float = distance_2d(0.0, 0.0, avg, avg)
    result: int = int(fmul(d, 10.0))
    print(result)
    assert result == 87


def test_loop_sqrt_accumulate() -> None:
    total: float = 0.0
    i: int = 1
    while i <= 5:
        total = fadd(total, newton_sqrt(float(i * i * i), 15))
        i = i + 1
    result: int = int(total)
    print(result)
    assert result == 28


def run_tests() -> None:
    test_fsquare_of_sum()
    test_fcube_nested()
    test_fabs_of_expression()
    test_fclamp_via_fmax_fmin()
    test_weighted_avg4_balanced()
    test_weighted_avg4_from_ops()
    test_lerp_quarter()
    test_inverse_lerp_roundtrip()
    test_remap_basic()
    test_remap_with_clamp()
    test_remap_from_computed_bounds()
    test_distance_2d_basic()
    test_distance_2d_from_ops()
    test_sum_float_range_with_step()
    test_sum_range_feeds_sqrt()
    test_fpower_basic()
    test_fpower_nested_args()
    test_newton_sqrt_of_computed()
    test_bisect_sqrt_of_16()
    test_bisect_sqrt_of_2_approx()
    test_geometric_series_basic()
    test_geometric_series_half()
    test_taylor_exp_at_1()
    test_taylor_exp_at_2()
    test_harmonic_10()
    test_harmonic_feeds_lerp()
    test_moving_average_chain()
    test_moving_average_loop()
    test_smooth_step_edges()
    test_smooth_step_mid()
    test_smooth_step_from_remap()
    test_mega_float_pipeline()
    test_convergent_float_branches()
    test_loop_sqrt_accumulate()
