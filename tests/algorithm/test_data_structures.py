def dsu_make_parent(n: int) -> list[int]:
    parent: list[int] = []
    i: int = 0
    while i < n:
        parent.append(i)
        i = i + 1
    return parent


def dsu_make_size(n: int) -> list[int]:
    size: list[int] = []
    i: int = 0
    while i < n:
        size.append(1)
        i = i + 1
    return size


def dsu_find(parent: list[int], x: int) -> int:
    cur: int = x
    while parent[cur] != cur:
        cur = parent[cur]

    root: int = cur
    cur = x
    while parent[cur] != cur:
        nxt: int = parent[cur]
        parent[cur] = root
        cur = nxt
    return root


def dsu_union(parent: list[int], size: list[int], a: int, b: int) -> bool:
    ra: int = dsu_find(parent, a)
    rb: int = dsu_find(parent, b)
    if ra == rb:
        return False
    if size[ra] < size[rb]:
        t: int = ra
        ra = rb
        rb = t
    parent[rb] = ra
    size[ra] = size[ra] + size[rb]
    return True


def heap_push(heap: list[int], value: int) -> None:
    heap.append(value)
    i: int = len(heap) - 1
    while i > 0:
        p: int = (i - 1) // 2
        if heap[p] <= heap[i]:
            break
        t: int = heap[p]
        heap[p] = heap[i]
        heap[i] = t
        i = p


def heap_pop(heap: list[int]) -> int:
    top: int = heap[0]
    last: int = heap[len(heap) - 1]
    heap[0] = last
    heap.pop()  # del heap[len(heap) - 1]

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
    return top


def test_dsu_large_components() -> None:
    n: int = 6000
    parent: list[int] = dsu_make_parent(n)
    size: list[int] = dsu_make_size(n)

    i: int = 1
    while i < n:
        dsu_union(parent, size, i - 1, i)
        i = i + 1

    root0: int = dsu_find(parent, 0)
    assert size[root0] == n

    j: int = 0
    while j < n:
        assert dsu_find(parent, j) == root0
        j = j + 1

    print(size[root0])


def test_heap_priority_queue_large() -> None:
    n: int = 8000
    value_mod: int = 2048
    heap: list[int] = []
    counts: list[int] = []
    i: int = 0
    while i < value_mod:
        counts.append(0)
        i = i + 1

    k: int = 0
    while k < n:
        v: int = (k * 97 + 31) % value_mod
        counts[v] = counts[v] + 1
        heap_push(heap, v)
        k = k + 1

    produced: int = 0
    expected_value: int = 0
    while produced < n:
        while counts[expected_value] == 0:
            expected_value = expected_value + 1
        popped: int = heap_pop(heap)
        assert popped == expected_value
        counts[expected_value] = counts[expected_value] - 1
        produced = produced + 1

    assert len(heap) == 0
    print(expected_value)


def run_tests() -> None:
    test_dsu_large_components()
    test_heap_priority_queue_large()
