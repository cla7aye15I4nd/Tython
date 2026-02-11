def gcd(a: int, b: int) -> int:
    x: int = a
    y: int = b
    while y != 0:
        t: int = x % y
        x = y
        y = t
    return x


def fib_dp(n: int) -> int:
    if n <= 1:
        return n

    dp: list[int] = []
    i: int = 0
    while i <= n:
        dp.append(0)
        i = i + 1

    dp[1] = 1
    i = 2
    while i <= n:
        dp[i] = dp[i - 1] + dp[i - 2]
        i = i + 1
    return dp[n]


def test_gcd_chain() -> None:
    xs: list[tuple[int, int, int]] = [
        (48, 18, 6),
        (270, 192, 6),
        (17, 13, 1),
        (100000, 250, 250),
    ]

    total: int = 0
    for item in xs:
        g: int = gcd(item[0], item[1])
        assert g == item[2]
        total = total + g
    assert total == 263


def test_prime_marking_nested_loops() -> None:
    n: int = 40
    is_prime: list[bool] = []
    i: int = 0
    while i <= n:
        is_prime.append(True)
        i = i + 1

    is_prime[0] = False
    is_prime[1] = False

    p: int = 2
    while p * p <= n:
        if is_prime[p]:
            k: int = p * p
            while k <= n:
                is_prime[k] = False
                k = k + p
        p = p + 1

    primes: list[int] = [x for x in range(2, n + 1) if is_prime[x]]
    assert primes == [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]

    s: int = 0
    for v in primes:
        s = s + v
    assert s == 197


def test_lis_quadratic_dp() -> None:
    arr: list[int] = [10, 9, 2, 5, 3, 7, 101, 18]
    n: int = len(arr)

    dp: list[int] = []
    i: int = 0
    while i < n:
        dp.append(1)
        i = i + 1

    i = 0
    while i < n:
        j: int = 0
        while j < i:
            if arr[j] < arr[i]:
                cand: int = dp[j] + 1
                if cand > dp[i]:
                    dp[i] = cand
            j = j + 1
        i = i + 1

    best: int = 0
    for v in dp:
        if v > best:
            best = v
    assert best == 4
    assert dp == [1, 1, 1, 2, 2, 3, 4, 4]


def test_knapsack_01_one_dimensional_dp() -> None:
    weights: list[int] = [2, 1, 3, 2]
    values: list[int] = [12, 10, 20, 15]
    cap: int = 5

    dp: list[int] = []
    c: int = 0
    while c <= cap:
        dp.append(0)
        c = c + 1

    i: int = 0
    while i < len(weights):
        w: int = weights[i]
        val: int = values[i]
        c = cap
        while c >= w:
            cand: int = dp[c - w] + val
            if cand > dp[c]:
                dp[c] = cand
            c = c - 1
        i = i + 1

    assert dp == [0, 10, 15, 25, 30, 37]
    assert dp[cap] == 37


def run_tests() -> None:
    test_gcd_chain()
    test_prime_marking_nested_loops()
    test_lis_quadratic_dp()
    test_knapsack_01_one_dimensional_dp()
