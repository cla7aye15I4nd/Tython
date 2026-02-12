class DisjointSetUnion:
    parent: list[int]
    size: list[int]

    def __init__(self, n: int) -> None:
        self.parent = []
        self.size = []
        i: int = 0
        while i < n:
            self.parent.append(i)
            self.size.append(1)
            i = i + 1

    def find(self, x: int) -> int:
        cur: int = x
        while self.parent[cur] != cur:
            cur = self.parent[cur]

        root: int = cur
        cur = x
        while self.parent[cur] != cur:
            nxt: int = self.parent[cur]
            self.parent[cur] = root
            cur = nxt
        return root

    def union(self, a: int, b: int) -> bool:
        ra: int = self.find(a)
        rb: int = self.find(b)
        if ra == rb:
            return False
        if self.size[ra] < self.size[rb]:
            t: int = ra
            ra = rb
            rb = t
        self.parent[rb] = ra
        self.size[ra] = self.size[ra] + self.size[rb]
        return True

    def component_size(self, x: int) -> int:
        return self.size[self.find(x)]


class MinHeap:
    data: list[int]

    def __init__(self) -> None:
        self.data = []

    def __len__(self) -> int:
        return len(self.data)

    def push(self, value: int) -> None:
        self.data.append(value)
        i: int = len(self.data) - 1
        while i > 0:
            p: int = (i - 1) // 2
            if self.data[p] <= self.data[i]:
                break
            t: int = self.data[p]
            self.data[p] = self.data[i]
            self.data[i] = t
            i = p

    def pop(self) -> int:
        top: int = self.data[0]
        last: int = self.data[len(self.data) - 1]
        self.data[0] = last
        self.data.pop()

        i: int = 0
        n: int = len(self.data)
        while True:
            left: int = i * 2 + 1
            if left >= n:
                break
            right: int = left + 1
            child: int = left
            if right < n and self.data[right] < self.data[left]:
                child = right
            if self.data[i] <= self.data[child]:
                break
            t: int = self.data[i]
            self.data[i] = self.data[child]
            self.data[child] = t
            i = child
        return top


def test_dsu_large_components_and_queries() -> None:
    n: int = 7000
    dsu: DisjointSetUnion = DisjointSetUnion(n)

    i: int = 0
    while i + 1 < n:
        dsu.union(i, i + 1)
        i = i + 2

    j: int = 0
    while j + 2 < n:
        dsu.union(j, j + 2)
        j = j + 4

    k: int = 1
    while k < n:
        dsu.union(0, k)
        k = k + 1

    root0: int = dsu.find(0)
    print('CHECK test_data_structures lhs:', dsu.size[root0])
    print('CHECK test_data_structures rhs:', n)
    assert dsu.size[root0] == n
    print('CHECK test_data_structures lhs:', dsu.component_size(17))
    print('CHECK test_data_structures rhs:', n)
    assert dsu.component_size(17) == n
    print('CHECK test_data_structures assert expr:', 'not dsu.union(3, 4)')
    assert not dsu.union(3, 4)

    x: int = 0
    while x < n:
        print('CHECK test_data_structures lhs:', dsu.find(x))
        print('CHECK test_data_structures rhs:', root0)
        assert dsu.find(x) == root0
        x = x + 1

    print(dsu.size[root0])


def test_heap_priority_queue_large_and_interleaved() -> None:
    n: int = 9000
    value_mod: int = 2500
    heap: MinHeap = MinHeap()
    counts: list[int] = []

    i: int = 0
    while i < value_mod:
        counts.append(0)
        i = i + 1

    k: int = 0
    while k < n:
        v: int = (k * 97 + 31) % value_mod
        counts[v] = counts[v] + 1
        heap.push(v)
        if k % 7 == 0:
            w: int = (k * 41 + 19) % value_mod
            counts[w] = counts[w] + 1
            heap.push(w)
        k = k + 1

    expected_total: int = n + (n + 6) // 7
    produced: int = 0
    expected_value: int = 0
    checksum: int = 0

    while produced < expected_total:
        while counts[expected_value] == 0:
            expected_value = expected_value + 1
        popped: int = heap.pop()
        print('CHECK test_data_structures lhs:', popped)
        print('CHECK test_data_structures rhs:', expected_value)
        assert popped == expected_value
        counts[expected_value] = counts[expected_value] - 1
        checksum = (checksum + popped * (produced + 1)) % 1000000007
        produced = produced + 1

    print('CHECK test_data_structures lhs:', len(heap))
    print('CHECK test_data_structures rhs:', 0)
    assert len(heap) == 0
    print('CHECK test_data_structures lhs:', expected_value)
    print('CHECK test_data_structures rhs:', value_mod - 1)
    assert expected_value == value_mod - 1
    print(checksum)


def run_tests() -> None:
    test_dsu_large_components_and_queries()
    test_heap_priority_queue_large_and_interleaved()
