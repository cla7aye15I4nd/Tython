def add_edge(head: list[int], to: list[int], nxt: list[int], indeg: list[int], u: int, v: int) -> None:
    to.append(v)
    nxt.append(head[u])
    head[u] = len(to) - 1
    indeg[v] = indeg[v] + 1


def topo_sort_kahn(n: int, head: list[int], to: list[int], nxt: list[int], indeg: list[int]) -> list[int]:
    q: list[int] = []
    i: int = 0
    while i < n:
        if indeg[i] == 0:
            q.append(i)
        i = i + 1

    order: list[int] = []
    qh: int = 0
    while qh < len(q):
        u: int = q[qh]
        qh = qh + 1
        order.append(u)

        e: int = head[u]
        while e != -1:
            v: int = to[e]
            indeg[v] = indeg[v] - 1
            if indeg[v] == 0:
                q.append(v)
            e = nxt[e]
    return order


def heap_push_pair(heap: list[int], dist: int, node: int, shift: int) -> None:
    key: int = dist * shift + node
    heap.append(key)
    i: int = len(heap) - 1
    while i > 0:
        p: int = (i - 1) // 2
        if heap[p] <= heap[i]:
            break
        t: int = heap[p]
        heap[p] = heap[i]
        heap[i] = t
        i = p


def heap_pop_pair_key(heap: list[int]) -> int:
    key: int = heap[0]
    last: int = heap[len(heap) - 1]
    heap[0] = last
    heap.pop()

    i: int = 0
    n: int = len(heap)
    while True:
        left: int = i * 2 + 1
        if left >= n:
            break
        right: int = left + 1
        child: int = left
        if right < n and heap[right] < heap[left]:
            child = right
        if heap[i] <= heap[child]:
            break
        t: int = heap[i]
        heap[i] = heap[child]
        heap[child] = t
        i = child

    return key


def test_topological_sort_large_dag() -> None:
    layers: int = 50
    width: int = 60
    n: int = layers * width

    head: list[int] = []
    indeg: list[int] = []
    i: int = 0
    while i < n:
        head.append(-1)
        indeg.append(0)
        i = i + 1

    to: list[int] = []
    nxt: list[int] = []

    layer: int = 0
    while layer + 1 < layers:
        a: int = 0
        while a < width:
            u: int = layer * width + a
            b: int = 0
            while b < width:
                v: int = (layer + 1) * width + b
                add_edge(head, to, nxt, indeg, u, v)
                b = b + 1
            a = a + 1
        layer = layer + 1

    order: list[int] = topo_sort_kahn(n, head, to, nxt, indeg)
    assert len(order) == n

    pos: list[int] = []
    j: int = 0
    while j < n:
        pos.append(0)
        j = j + 1
    k: int = 0
    while k < n:
        pos[order[k]] = k
        k = k + 1

    u2: int = 0
    while u2 < n:
        e: int = head[u2]
        while e != -1:
            v2: int = to[e]
            assert pos[u2] < pos[v2]
            e = nxt[e]
        u2 = u2 + 1

    print(len(to))


def test_dijkstra_large_weighted_grid() -> None:
    rows: int = 45
    cols: int = 45
    n: int = rows * cols
    inf: int = 1000000000

    dist: list[int] = []
    i: int = 0
    while i < n:
        dist.append(inf)
        i = i + 1

    shift: int = n + 1
    heap: list[int] = []
    dist[0] = 0
    heap_push_pair(heap, 0, 0, shift)

    while len(heap) > 0:
        key: int = heap_pop_pair_key(heap)
        d: int = key // shift
        u: int = key % shift
        if d != dist[u]:
            continue

        r: int = u // cols
        c: int = u % cols

        # Right neighbor.
        if c + 1 < cols:
            v: int = u + 1
            w: int = 1 + ((r + c) % 3)
            nd: int = d + w
            if nd < dist[v]:
                dist[v] = nd
                heap_push_pair(heap, nd, v, shift)

        # Down neighbor.
        if r + 1 < rows:
            v2: int = u + cols
            w2: int = 1 + ((r * 2 + c) % 4)
            nd2: int = d + w2
            if nd2 < dist[v2]:
                dist[v2] = nd2
                heap_push_pair(heap, nd2, v2, shift)

    target: int = n - 1
    assert dist[target] > 0

    # DP on same acyclic right/down graph for cross-check.
    dp: list[int] = []
    x: int = 0
    while x < n:
        dp.append(inf)
        x = x + 1
    dp[0] = 0

    rr: int = 0
    while rr < rows:
        cc: int = 0
        while cc < cols:
            u3: int = rr * cols + cc
            cur: int = dp[u3]
            if cur < inf:
                if cc + 1 < cols:
                    v3: int = u3 + 1
                    w3: int = 1 + ((rr + cc) % 3)
                    cand: int = cur + w3
                    if cand < dp[v3]:
                        dp[v3] = cand
                if rr + 1 < rows:
                    v4: int = u3 + cols
                    w4: int = 1 + ((rr * 2 + cc) % 4)
                    cand2: int = cur + w4
                    if cand2 < dp[v4]:
                        dp[v4] = cand2
            cc = cc + 1
        rr = rr + 1

    assert dist[target] == dp[target]
    print(dist[target])


def run_tests() -> None:
    test_topological_sort_large_dag()
    test_dijkstra_large_weighted_grid()
