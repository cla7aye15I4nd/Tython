def gcd_int(a: int, b: int) -> int:
    x: int = a
    y: int = b
    while y != 0:
        t: int = x % y
        x = y
        y = t
    if x < 0:
        return -x
    return x


def ext_gcd(a: int, b: int) -> list[int]:
    old_r: int = a
    r: int = b
    old_s: int = 1
    s: int = 0
    old_t: int = 0
    t: int = 1

    while r != 0:
        q: int = old_r // r

        tmp_r: int = old_r - q * r
        old_r = r
        r = tmp_r

        tmp_s: int = old_s - q * s
        old_s = s
        s = tmp_s

        tmp_t: int = old_t - q * t
        old_t = t
        t = tmp_t

    if old_r < 0:
        return [-old_r, -old_s, -old_t]
    return [old_r, old_s, old_t]


def mod_pow(base: int, exp: int, mod: int) -> int:
    result: int = 1
    b: int = base % mod
    e: int = exp
    while e > 0:
        if e % 2 == 1:
            result = (result * b) % mod
        b = (b * b) % mod
        e = e // 2
    return result


def mod_pow_slow(base: int, exp: int, mod: int) -> int:
    result: int = 1
    i: int = 0
    while i < exp:
        result = (result * base) % mod
        i = i + 1
    return result


def pascal_row_mod(n: int, mod: int) -> list[int]:
    row: list[int] = []
    i: int = 0
    while i <= n:
        row.append(0)
        i = i + 1
    row[0] = 1

    r: int = 1
    while r <= n:
        c: int = r
        while c > 0:
            row[c] = (row[c] + row[c - 1]) % mod
            c = c - 1
        r = r + 1
    return row


def test_extended_gcd_batch() -> None:
    i: int = 1
    checked: int = 0
    while i <= 1200:
        a: int = i * 137 + 53
        b: int = i * 89 + 17
        eg: list[int] = ext_gcd(a, b)
        g: int = eg[0]
        x: int = eg[1]
        y: int = eg[2]

        print('CHECK test_math_algorithms lhs:', a * x + b * y)
        print('CHECK test_math_algorithms rhs:', g)
        assert a * x + b * y == g
        print('CHECK test_math_algorithms lhs:', g)
        print('CHECK test_math_algorithms rhs expr:', 'gcd_int(a, b)')
        assert g == gcd_int(a, b)

        checked = checked + 1
        i = i + 1

    print(checked)


def test_fast_mod_pow_many_queries() -> None:
    mod: int = 1000000007
    q: int = 0
    acc: int = 0
    while q < 500:
        base: int = (q * 97 + 11) % mod
        exp: int = (q * 13 + 5) % 250

        fast: int = mod_pow(base, exp, mod)
        slow: int = mod_pow_slow(base, exp, mod)
        print('CHECK test_math_algorithms lhs:', fast)
        print('CHECK test_math_algorithms rhs:', slow)
        assert fast == slow

        acc = (acc + fast) % mod
        q = q + 1

    print(acc)


def test_pascal_row_mod_large() -> None:
    mod: int = 1000000007
    n: int = 800
    row: list[int] = pascal_row_mod(n, mod)
    print('CHECK test_math_algorithms lhs expr:', 'len(row)')
    print('CHECK test_math_algorithms rhs:', n + 1)
    assert len(row) == n + 1

    # Symmetry: C(n, k) == C(n, n-k).
    k: int = 0
    while k <= n:
        print('CHECK test_math_algorithms lhs:', row[k])
        print('CHECK test_math_algorithms rhs:', row[n - k])
        assert row[k] == row[n - k]
        k = k + 1

    # Sum of row is 2^n mod mod.
    total: int = 0
    i: int = 0
    while i <= n:
        total = (total + row[i]) % mod
        i = i + 1
    print('CHECK test_math_algorithms lhs:', total)
    print('CHECK test_math_algorithms rhs expr:', 'mod_pow(2, n, mod)')
    assert total == mod_pow(2, n, mod)

    # Spot checks with known small combinations.
    row20: list[int] = pascal_row_mod(20, mod)
    print('CHECK test_math_algorithms lhs:', row20[10])
    print('CHECK test_math_algorithms rhs:', 184756)
    assert row20[10] == 184756
    print('CHECK test_math_algorithms lhs:', row20[3])
    print('CHECK test_math_algorithms rhs:', 1140)
    assert row20[3] == 1140

    print(row[n // 2])


def run_tests() -> None:
    test_extended_gcd_batch()
    test_fast_mod_pow_many_queries()
    test_pascal_row_mod_large()
