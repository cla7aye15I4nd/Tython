def build_int_signal(n: int) -> list[int]:
    values: list[int] = []
    i: int = 0
    while i < n:
        # Deterministic, non-trivial pattern in [0, 996].
        values.append((i * 17 + 23) % 997)
        i = i + 1
    return values


def build_prefix_sums(values: list[int]) -> list[int]:
    prefix: list[int] = []
    running: int = 0
    i: int = 0
    while i < len(values):
        running = running + values[i]
        prefix.append(running)
        i = i + 1
    return prefix


def range_sum(prefix: list[int], left: int, right: int) -> int:
    if left == 0:
        return prefix[right]
    return prefix[right] - prefix[left - 1]


def build_float_signal(n: int) -> list[float]:
    values: list[float] = []
    i: int = 0
    while i < n:
        base: float = float((i * 29 + 7) % 113)
        wobble: float = float((i * 11 + 3) % 17) / 100.0
        values.append(base + wobble)
        i = i + 1
    return values


def cumulative_mean(values: list[float]) -> list[float]:
    means: list[float] = []
    total: float = 0.0
    i: int = 0
    while i < len(values):
        total = total + values[i]
        means.append(total / float(i + 1))
        i = i + 1
    return means


def sieve_primes(limit: int) -> list[bool]:
    is_prime: list[bool] = []
    i: int = 0
    while i <= limit:
        is_prime.append(True)
        i = i + 1

    if limit >= 0:
        is_prime[0] = False
    if limit >= 1:
        is_prime[1] = False

    p: int = 2
    while p * p <= limit:
        if is_prime[p]:
            multiple: int = p * p
            while multiple <= limit:
                is_prime[multiple] = False
                multiple = multiple + p
        p = p + 1
    return is_prime


def count_true(values: list[bool]) -> int:
    count: int = 0
    i: int = 0
    while i < len(values):
        if values[i]:
            count = count + 1
        i = i + 1
    return count


def direct_range_sum(values: list[int], left: int, right: int) -> int:
    total: int = 0
    i: int = left
    while i <= right:
        total = total + values[i]
        i = i + 1
    return total


def test_large_prefix_and_range_queries() -> None:
    n: int = 8000
    values: list[int] = build_int_signal(n)
    prefix: list[int] = build_prefix_sums(values)

    print('CHECK test_list_algorithms lhs:', len(values))
    print('CHECK test_list_algorithms rhs:', n)
    assert len(values) == n
    print('CHECK test_list_algorithms lhs:', len(prefix))
    print('CHECK test_list_algorithms rhs:', n)
    assert len(prefix) == n

    print('CHECK test_list_algorithms lhs:', range_sum(prefix, 0, 7999))
    print('CHECK test_list_algorithms rhs:', direct_range_sum(values, 0, 7999))
    assert range_sum(prefix, 0, 7999) == direct_range_sum(values, 0, 7999)
    print('CHECK test_list_algorithms lhs:', range_sum(prefix, 123, 3456))
    print('CHECK test_list_algorithms rhs:', direct_range_sum(values, 123, 3456))
    assert range_sum(prefix, 123, 3456) == direct_range_sum(values, 123, 3456)
    print('CHECK test_list_algorithms lhs:', range_sum(prefix, 2000, 7000))
    print('CHECK test_list_algorithms rhs:', direct_range_sum(values, 2000, 7000))
    assert range_sum(prefix, 2000, 7000) == direct_range_sum(values, 2000, 7000)
    print('CHECK test_list_algorithms lhs:', range_sum(prefix, 7999, 7999))
    print('CHECK test_list_algorithms rhs:', values[7999])
    assert range_sum(prefix, 7999, 7999) == values[7999]

    print(prefix[7999])


def test_large_cumulative_mean() -> None:
    n: int = 6000
    values: list[float] = build_float_signal(n)
    means: list[float] = cumulative_mean(values)

    print('CHECK test_list_algorithms lhs:', len(means))
    print('CHECK test_list_algorithms rhs:', n)
    assert len(means) == n

    total: float = 0.0
    i: int = 0
    while i < n:
        total = total + values[i]
        i = i + 1

    last_mean: float = means[n - 1]
    expected_last: float = total / float(n)
    print('CHECK test_list_algorithms assert expr:', 'abs(last_mean - expected_last) < 0.000001')
    assert abs(last_mean - expected_last) < 0.000001

    spot: int = 4999
    partial: float = 0.0
    j: int = 0
    while j <= spot:
        partial = partial + values[j]
        j = j + 1
    print('CHECK test_list_algorithms assert expr:', 'abs(means[spot] - (partial / float(spot + 1))) < 0.000001')
    assert abs(means[spot] - (partial / float(spot + 1))) < 0.000001

    print(int(last_mean))


def test_large_sieve_with_bool_list() -> None:
    limit: int = 10000
    is_prime: list[bool] = sieve_primes(limit)
    prime_count: int = count_true(is_prime)

    print('CHECK test_list_algorithms lhs:', prime_count)
    print('CHECK test_list_algorithms rhs:', 1229)
    assert prime_count == 1229
    print('CHECK test_list_algorithms lhs:', is_prime[2])
    print('CHECK test_list_algorithms rhs:', True)
    assert is_prime[2] == True
    print('CHECK test_list_algorithms lhs:', is_prime[97])
    print('CHECK test_list_algorithms rhs:', True)
    assert is_prime[97] == True
    print('CHECK test_list_algorithms lhs:', is_prime[9999])
    print('CHECK test_list_algorithms rhs:', False)
    assert is_prime[9999] == False
    print('CHECK test_list_algorithms lhs:', is_prime[10000])
    print('CHECK test_list_algorithms rhs:', False)
    assert is_prime[10000] == False

    print(prime_count)


def fenwick_add(tree: list[int], n: int, index_1based: int, delta: int) -> None:
    i: int = index_1based
    while i <= n:
        tree[i] = tree[i] + delta
        i = i + (i & -i)


def fenwick_sum(tree: list[int], index_1based: int) -> int:
    result: int = 0
    i: int = index_1based
    while i > 0:
        result = result + tree[i]
        i = i - (i & -i)
    return result


def fenwick_range_sum(tree: list[int], left_1based: int, right_1based: int) -> int:
    if left_1based == 1:
        return fenwick_sum(tree, right_1based)
    return fenwick_sum(tree, right_1based) - fenwick_sum(tree, left_1based - 1)


def build_grid_graph(rows: int, cols: int, head: list[int], to: list[int], nxt: list[int]) -> None:
    r: int = 0
    while r < rows:
        c: int = 0
        while c < cols:
            u: int = r * cols + c

            if c + 1 < cols:
                v: int = r * cols + (c + 1)
                to.append(v)
                nxt.append(head[u])
                head[u] = len(to) - 1

                to.append(u)
                nxt.append(head[v])
                head[v] = len(to) - 1

            if r + 1 < rows:
                w: int = (r + 1) * cols + c
                to.append(w)
                nxt.append(head[u])
                head[u] = len(to) - 1

                to.append(u)
                nxt.append(head[w])
                head[w] = len(to) - 1

            c = c + 1
        r = r + 1


def bfs_distances(n: int, head: list[int], to: list[int], nxt: list[int], start: int) -> list[int]:
    dist: list[int] = []
    i: int = 0
    while i < n:
        dist.append(-1)
        i = i + 1

    queue: list[int] = []
    dist[start] = 0
    queue.append(start)

    qh: int = 0
    while qh < len(queue):
        u: int = queue[qh]
        qh = qh + 1

        e: int = head[u]
        while e != -1:
            v: int = to[e]
            if dist[v] == -1:
                dist[v] = dist[u] + 1
                queue.append(v)
            e = nxt[e]

    return dist


def mat_mul_mod_2x2(a: list[int], b: list[int], mod: int) -> list[int]:
    # Layout: [a00, a01, a10, a11].
    c00: int = (a[0] * b[0] + a[1] * b[2]) % mod
    c01: int = (a[0] * b[1] + a[1] * b[3]) % mod
    c10: int = (a[2] * b[0] + a[3] * b[2]) % mod
    c11: int = (a[2] * b[1] + a[3] * b[3]) % mod
    return [c00, c01, c10, c11]


def fib_matrix_mod(n: int, mod: int) -> int:
    if n == 0:
        return 0

    # Identity matrix.
    result: list[int] = [1, 0, 0, 1]
    # Fibonacci transition matrix.
    base: list[int] = [1, 1, 1, 0]
    exp: int = n - 1

    while exp > 0:
        if exp % 2 == 1:
            result = mat_mul_mod_2x2(result, base, mod)
        base = mat_mul_mod_2x2(base, base, mod)
        exp = exp // 2

    # F(n) is in position [0][0] when multiplied by vector [F1, F0].
    return result[0] % mod


def fib_linear_mod(n: int, mod: int) -> int:
    if n == 0:
        return 0
    if n == 1:
        return 1

    a: int = 0
    b: int = 1
    i: int = 2
    while i <= n:
        nxt_val: int = (a + b) % mod
        a = b
        b = nxt_val
        i = i + 1
    return b


def test_fenwick_tree_large_scale() -> None:
    n: int = 10000
    tree: list[int] = []
    values: list[int] = []

    i: int = 0
    while i <= n:
        tree.append(0)
        i = i + 1

    j: int = 0
    while j < n:
        values.append(0)
        j = j + 1

    k: int = 0
    while k < 7000:
        idx0: int = (k * 37 + 11) % n
        delta: int = ((k * 13 + 19) % 41) - 20
        values[idx0] = values[idx0] + delta
        fenwick_add(tree, n, idx0 + 1, delta)
        k = k + 1

    prefix_naive: list[int] = []
    running: int = 0
    t: int = 0
    while t < n:
        running = running + values[t]
        prefix_naive.append(running)
        t = t + 1

    q: int = 0
    while q < 150:
        left0: int = (q * 71 + 5) % n
        width: int = (q * 29 + 101) % (n - left0)
        right0: int = left0 + width

        fenwick_ans: int = fenwick_range_sum(tree, left0 + 1, right0 + 1)
        naive_ans: int = prefix_naive[right0]
        if left0 > 0:
            naive_ans = naive_ans - prefix_naive[left0 - 1]
        print('CHECK test_list_algorithms lhs:', fenwick_ans)
        print('CHECK test_list_algorithms rhs:', naive_ans)
        assert fenwick_ans == naive_ans
        q = q + 1

    print(fenwick_sum(tree, n))


def test_graph_bfs_large_grid() -> None:
    rows: int = 70
    cols: int = 70
    n: int = rows * cols
    head: list[int] = []
    i: int = 0
    while i < n:
        head.append(-1)
        i = i + 1
    to: list[int] = []
    nxt: list[int] = []
    build_grid_graph(rows, cols, head, to, nxt)

    dist: list[int] = bfs_distances(n, head, to, nxt, 0)
    target: int = n - 1

    # Manhattan distance from (0,0) to (rows-1, cols-1).
    print('CHECK test_list_algorithms lhs:', dist[target])
    print('CHECK test_list_algorithms rhs:', rows - 1 + (cols - 1))
    assert dist[target] == (rows - 1) + (cols - 1)
    print('CHECK test_list_algorithms lhs:', dist[1])
    print('CHECK test_list_algorithms rhs:', 1)
    assert dist[1] == 1
    print('CHECK test_list_algorithms lhs:', dist[cols])
    print('CHECK test_list_algorithms rhs:', 1)
    assert dist[cols] == 1
    print('CHECK test_list_algorithms lhs:', dist[rows // 2 * cols + cols // 2])
    print('CHECK test_list_algorithms rhs:', rows // 2 + cols // 2)
    assert dist[(rows // 2) * cols + (cols // 2)] == (rows // 2) + (cols // 2)

    print(dist[target])


def test_fibonacci_matrix_exponentiation_large_n() -> None:
    mod: int = 1000000007
    n: int = 50000

    fast_val: int = fib_matrix_mod(n, mod)
    linear_val: int = fib_linear_mod(n, mod)
    print('CHECK test_list_algorithms lhs:', fast_val)
    print('CHECK test_list_algorithms rhs:', linear_val)
    assert fast_val == linear_val

    print('CHECK test_list_algorithms lhs:', fib_matrix_mod(0, mod))
    print('CHECK test_list_algorithms rhs:', 0)
    assert fib_matrix_mod(0, mod) == 0
    print('CHECK test_list_algorithms lhs:', fib_matrix_mod(1, mod))
    print('CHECK test_list_algorithms rhs:', 1)
    assert fib_matrix_mod(1, mod) == 1
    print('CHECK test_list_algorithms lhs:', fib_matrix_mod(10, mod))
    print('CHECK test_list_algorithms rhs:', 55)
    assert fib_matrix_mod(10, mod) == 55

    print(fast_val)


def run_tests() -> None:
    test_large_prefix_and_range_queries()
    test_large_cumulative_mean()
    test_large_sieve_with_bool_list()
    test_fenwick_tree_large_scale()
    test_graph_bfs_large_grid()
    test_fibonacci_matrix_exponentiation_large_n()
