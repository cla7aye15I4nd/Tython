class FenwickTree:
    n: int
    bit: list[int]

    def __init__(self, n: int) -> None:
        self.n = n
        self.bit = []
        i: int = 0
        while i <= n:
            self.bit.append(0)
            i = i + 1

    def add(self, idx_1based: int, delta: int) -> None:
        i: int = idx_1based
        while i <= self.n:
            self.bit[i] = self.bit[i] + delta
            i = i + (i & -i)

    def prefix_sum(self, idx_1based: int) -> int:
        res: int = 0
        i: int = idx_1based
        while i > 0:
            res = res + self.bit[i]
            i = i - (i & -i)
        return res

    def range_sum(self, l_1based: int, r_1based: int) -> int:
        if l_1based == 1:
            return self.prefix_sum(r_1based)
        return self.prefix_sum(r_1based) - self.prefix_sum(l_1based - 1)


class SegmentTreeSum:
    n: int
    size: int
    tree: list[int]

    def __init__(self, n: int) -> None:
        self.n = n
        self.size = 1
        while self.size < n:
            self.size = self.size * 2

        self.tree = []
        i: int = 0
        while i < self.size * 2:
            self.tree.append(0)
            i = i + 1

    def set_value(self, idx_0based: int, value: int) -> None:
        p: int = self.size + idx_0based
        self.tree[p] = value
        p = p // 2
        while p >= 1:
            self.tree[p] = self.tree[p * 2] + self.tree[p * 2 + 1]
            p = p // 2

    def add_value(self, idx_0based: int, delta: int) -> None:
        p: int = self.size + idx_0based
        self.tree[p] = self.tree[p] + delta
        p = p // 2
        while p >= 1:
            self.tree[p] = self.tree[p * 2] + self.tree[p * 2 + 1]
            p = p // 2

    def range_sum(self, l_0based: int, r_0based: int) -> int:
        l: int = l_0based + self.size
        r: int = r_0based + self.size
        res: int = 0

        while l <= r:
            if l % 2 == 1:
                res = res + self.tree[l]
                l = l + 1
            if r % 2 == 0:
                res = res + self.tree[r]
                r = r - 1
            l = l // 2
            r = r // 2
        return res


def test_fenwick_tree_massive_updates_and_queries() -> None:
    n: int = 12000
    fw: FenwickTree = FenwickTree(n)
    values: list[int] = []
    i: int = 0
    while i < n:
        values.append(0)
        i = i + 1

    k: int = 0
    while k < 20000:
        idx0: int = (k * 73 + 19) % n
        delta: int = ((k * 17 + 29) % 101) - 50
        values[idx0] = values[idx0] + delta
        fw.add(idx0 + 1, delta)
        k = k + 1

    q: int = 0
    checksum: int = 0
    while q < 2000:
        l0: int = (q * 37 + 11) % n
        width: int = (q * 53 + 7) % 400
        r0: int = l0 + width
        if r0 >= n:
            r0 = n - 1

        naive: int = 0
        x: int = l0
        while x <= r0:
            naive = naive + values[x]
            x = x + 1

        got: int = fw.range_sum(l0 + 1, r0 + 1)
        assert got == naive
        checksum = (checksum + (got % 1000000007 + 1000000007) % 1000000007) % 1000000007
        q = q + 1

    print(checksum)


def test_segment_tree_sum_interleaved_ops() -> None:
    n: int = 8192
    st: SegmentTreeSum = SegmentTreeSum(n)
    values: list[int] = []

    i: int = 0
    while i < n:
        v: int = ((i * 41 + 23) % 1000) - 500
        values.append(v)
        st.set_value(i, v)
        i = i + 1

    op: int = 0
    while op < 10000:
        if op % 3 == 0:
            idx: int = (op * 61 + 5) % n
            nv: int = ((op * 19 + 7) % 2000) - 1000
            values[idx] = nv
            st.set_value(idx, nv)
        else:
            idx2: int = (op * 47 + 13) % n
            delta: int = ((op * 31 + 3) % 21) - 10
            values[idx2] = values[idx2] + delta
            st.add_value(idx2, delta)
        op = op + 1

    q: int = 0
    agg: int = 0
    while q < 2500:
        l: int = (q * 89 + 17) % n
        width2: int = (q * 67 + 9) % 600
        r: int = l + width2
        if r >= n:
            r = n - 1

        naive2: int = 0
        j: int = l
        while j <= r:
            naive2 = naive2 + values[j]
            j = j + 1

        got2: int = st.range_sum(l, r)
        assert got2 == naive2
        agg = (agg + (got2 % 1000000007 + 1000000007) % 1000000007) % 1000000007
        q = q + 1

    print(agg)


def run_tests() -> None:
    test_fenwick_tree_massive_updates_and_queries()
    test_segment_tree_sum_interleaved_ops()
