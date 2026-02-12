def add(a: int, b: int) -> int:
    return a + b


def sub(a: int, b: int) -> int:
    return a - b


def mul(a: int, b: int) -> int:
    return a * b


def mod(a: int, b: int) -> int:
    return a % b


def square(n: int) -> int:
    return mul(n, n)


def cube(n: int) -> int:
    return mul(square(n), n)


def abs_val(x: int) -> int:
    if x < 0:
        return sub(0, x)
    else:
        return x


def max2(a: int, b: int) -> int:
    if a >= b:
        return a
    else:
        return b


def min2(a: int, b: int) -> int:
    if a <= b:
        return a
    else:
        return b


def clamp(val: int, lo: int, hi: int) -> int:
    return max2(lo, min2(val, hi))


def weighted_sum4(a: int, wa: int, b: int, wb: int) -> int:
    return add(mul(a, wa), mul(b, wb))


def sum_range(lo: int, hi: int, step: int) -> int:
    total: int = 0
    i: int = lo
    while i <= hi:
        total = add(total, i)
        i = add(i, step)
    return total


def factorial(n: int) -> int:
    result: int = 1
    i: int = 2
    while i <= n:
        result = mul(result, i)
        i = add(i, 1)
    return result


def power(base: int, exp: int) -> int:
    result: int = 1
    i: int = 0
    while i < exp:
        result = mul(result, base)
        i = add(i, 1)
    return result


def gcd(a: int, b: int) -> int:
    x: int = abs_val(a)
    y: int = abs_val(b)
    while y != 0:
        temp: int = y
        y = mod(x, y)
        x = temp
    return x


def lcm(a: int, b: int) -> int:
    g: int = gcd(a, b)
    if g == 0:
        return 0
    else:
        return mul(a // g, b)


def collatz_steps(n: int) -> int:
    steps: int = 0
    val: int = n
    while val != 1:
        if mod(val, 2) == 0:
            val = val // 2
        else:
            val = add(mul(val, 3), 1)
        steps = add(steps, 1)
    return steps


def digit_sum(n: int) -> int:
    val: int = abs_val(n)
    total: int = 0
    while val > 0:
        total = add(total, mod(val, 10))
        val = val // 10
    return total


def count_divisors(n: int) -> int:
    count: int = 0
    i: int = 1
    while i <= n:
        if mod(n, i) == 0:
            count = add(count, 1)
        i = add(i, 1)
    return count


def is_prime_int(n: int) -> int:
    if n < 2:
        return 0
    i: int = 2
    while mul(i, i) <= n:
        if mod(n, i) == 0:
            return 0
        i = add(i, 1)
    return 1


def nth_prime(n: int) -> int:
    count: int = 0
    candidate: int = 2
    while count < n:
        if is_prime_int(candidate) == 1:
            count = add(count, 1)
        if count < n:
            candidate = add(candidate, 1)
    return candidate


def fibonacci(n: int) -> int:
    if n <= 1:
        return n
    a: int = 0
    b: int = 1
    i: int = 2
    while i <= n:
        temp: int = add(a, b)
        a = b
        b = temp
        i = add(i, 1)
    return b


def map_accumulate(start: int, count: int, mult: int, offset: int) -> int:
    acc: int = start
    i: int = 0
    while i < count:
        acc = add(mul(acc, mult), offset)
        i = add(i, 1)
    return acc


def triangular(n: int) -> int:
    return sum_range(1, n, 1)


def choose(n: int, k: int) -> int:
    if k > n:
        return 0
    if k == 0:
        return 1
    if k > sub(n, k):
        k = sub(n, k)
    result: int = 1
    i: int = 0
    while i < k:
        result = mul(result, sub(n, i))
        result = result // add(i, 1)
        i = add(i, 1)
    return result


# ── tests ─────────────────────────────────────────────────────────


def test_square_of_sum() -> None:
    s: int = add(3, 4)
    result: int = square(s)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 49)
    assert result == 49


def test_cube_from_nested() -> None:
    base: int = sub(10, 7)
    result: int = cube(base)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 27)
    assert result == 27


def test_abs_diff_via_abs() -> None:
    d: int = abs_val(sub(15, 42))
    print('CHECK test_int lhs:', d)
    print('CHECK test_int rhs:', 27)
    assert d == 27


def test_clamp_uses_min_max() -> None:
    raw: int = mul(add(8, 7), 12)
    result: int = clamp(raw, 50, 150)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 150)
    assert result == 150


def test_clamp_lower_bound() -> None:
    raw: int = sub(10, cube(3))
    result: int = clamp(raw, 0, 100)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 0)
    assert result == 0


def test_weighted_sum4_basic() -> None:
    result: int = weighted_sum4(3, 10, 7, 5)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 65)
    assert result == 65


def test_weighted_sum4_composed() -> None:
    a: int = square(3)
    b: int = cube(2)
    result: int = weighted_sum4(a, 4, b, 3)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 60)
    assert result == 60


def test_sum_range_with_step() -> None:
    result: int = sum_range(1, 20, 3)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 70)
    assert result == 70


def test_sum_range_feeds_square() -> None:
    s: int = sum_range(1, 5, 1)
    result: int = square(s)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 225)
    assert result == 225


def test_factorial_feeds_digit_sum() -> None:
    f: int = factorial(7)
    ds: int = digit_sum(f)
    print('CHECK test_int lhs:', ds)
    print('CHECK test_int rhs:', 9)
    assert ds == 9


def test_power_nested_args() -> None:
    base: int = add(1, sub(5, 3))
    exp: int = min2(square(2), 5)
    result: int = power(base, exp)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 81)
    assert result == 81


def test_gcd_chain() -> None:
    g1: int = gcd(mul(12, 7), mul(12, 5))
    g2: int = gcd(mul(9, 11), mul(9, 7))
    result: int = gcd(g1, g2)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 3)
    assert result == 3


def test_lcm_basic() -> None:
    result: int = lcm(12, 18)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 36)
    assert result == 36


def test_lcm_from_primes() -> None:
    p1: int = nth_prime(4)
    p2: int = nth_prime(5)
    result: int = lcm(p1, p2)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 77)
    assert result == 77


def test_collatz_known_values() -> None:
    s27: int = collatz_steps(27)
    s19: int = collatz_steps(19)
    result: int = add(s27, s19)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 131)
    assert result == 131


def test_collatz_feeds_clamp() -> None:
    steps: int = collatz_steps(27)
    result: int = clamp(steps, 0, 100)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 100)
    assert result == 100


def test_digit_sum_of_power() -> None:
    p: int = power(2, 15)
    result: int = digit_sum(p)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 26)
    assert result == 26


def test_digit_sum_of_factorial() -> None:
    f: int = factorial(10)
    result: int = digit_sum(f)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 27)
    assert result == 27


def test_count_divisors_basic() -> None:
    result: int = count_divisors(28)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 6)
    assert result == 6


def test_count_divisors_of_power() -> None:
    p: int = power(2, 5)
    result: int = count_divisors(p)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 6)
    assert result == 6


def test_is_prime_chain() -> None:
    total: int = 0
    i: int = 2
    while i <= 30:
        total = add(total, is_prime_int(i))
        i = add(i, 1)
    print('CHECK test_int lhs:', total)
    print('CHECK test_int rhs:', 10)
    assert total == 10


def test_nth_prime_values() -> None:
    p10: int = nth_prime(10)
    p5: int = nth_prime(5)
    result: int = sub(p10, p5)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 18)
    assert result == 18


def test_fibonacci_chain() -> None:
    a: int = fibonacci(10)
    b: int = fibonacci(8)
    result: int = gcd(a, b)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 1)
    assert result == 1


def test_fibonacci_sum_loop() -> None:
    total: int = 0
    i: int = 1
    while i <= 10:
        total = add(total, fibonacci(i))
        i = add(i, 1)
    result: int = mod(total, 100)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 43)
    assert result == 43


def test_map_accumulate_linear() -> None:
    result: int = map_accumulate(1, 5, 2, 1)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 63)
    assert result == 63


def test_map_accumulate_feeds_gcd() -> None:
    a: int = map_accumulate(1, 4, 3, 2)
    b: int = map_accumulate(2, 3, 3, 1)
    result: int = gcd(a, b)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 1)
    assert result == 1


def test_triangular_identity() -> None:
    t10: int = triangular(10)
    direct: int = mul(10, 11) // 2
    result: int = sub(t10, direct)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 0)
    assert result == 0


def test_choose_basic() -> None:
    result: int = choose(10, 3)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 120)
    assert result == 120


def test_choose_symmetry() -> None:
    a: int = choose(12, 4)
    b: int = choose(12, 8)
    result: int = sub(a, b)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 0)
    assert result == 0


def test_mega_pipeline() -> None:
    p7: int = nth_prime(7)
    fib_p: int = fibonacci(p7)
    ds: int = digit_sum(fib_p)
    g: int = gcd(ds, factorial(4))
    steps: int = collatz_steps(g)
    base: int = clamp(steps, 1, 10)
    result: int = power(base, 2)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 1)
    assert result == 1


def test_convergent_branches() -> None:
    left: int = digit_sum(power(3, 7))
    right: int = collatz_steps(mul(2, 7))
    mid: int = weighted_sum4(left, 3, right, 2)
    clamped: int = clamp(mid, 10, 80)
    result: int = mod(clamped, nth_prime(5))
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 3)
    assert result == 3


def test_loop_with_deep_calls() -> None:
    acc: int = 0
    i: int = 2
    while i <= 8:
        if is_prime_int(i) == 1:
            acc = add(acc, square(fibonacci(i)))
        else:
            acc = add(acc, digit_sum(cube(i)))
        i = add(i, 1)
    result: int = mod(acc, 100)
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 26)
    assert result == 26


def test_nested_choose_sum() -> None:
    total: int = 0
    k: int = 0
    while k <= 5:
        total = add(total, choose(5, k))
        k = add(k, 1)
    print('CHECK test_int lhs:', total)
    print('CHECK test_int rhs:', 32)
    assert total == 32


def test_double_accumulation() -> None:
    product: int = 1
    sum_val: int = 0
    i: int = 1
    while i <= 6:
        f: int = fibonacci(i)
        product = mul(product, max2(f, 1))
        sum_val = add(sum_val, square(f))
        i = add(i, 1)
    result: int = mod(product, add(sum_val, 1))
    print('CHECK test_int lhs:', result)
    print('CHECK test_int rhs:', 30)
    assert result == 30


def run_tests() -> None:
    test_square_of_sum()
    test_cube_from_nested()
    test_abs_diff_via_abs()
    test_clamp_uses_min_max()
    test_clamp_lower_bound()
    test_weighted_sum4_basic()
    test_weighted_sum4_composed()
    test_sum_range_with_step()
    test_sum_range_feeds_square()
    test_factorial_feeds_digit_sum()
    test_power_nested_args()
    test_gcd_chain()
    test_lcm_basic()
    test_lcm_from_primes()
    test_collatz_known_values()
    test_collatz_feeds_clamp()
    test_digit_sum_of_power()
    test_digit_sum_of_factorial()
    test_count_divisors_basic()
    test_count_divisors_of_power()
    test_is_prime_chain()
    test_nth_prime_values()
    test_fibonacci_chain()
    test_fibonacci_sum_loop()
    test_map_accumulate_linear()
    test_map_accumulate_feeds_gcd()
    test_triangular_identity()
    test_choose_basic()
    test_choose_symmetry()
    test_mega_pipeline()
    test_convergent_branches()
    test_loop_with_deep_calls()
    test_nested_choose_sum()
    test_double_accumulation()
