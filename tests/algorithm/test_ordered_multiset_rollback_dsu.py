class Fenwick:
    n: int
    bit: list[int]

    def __init__(self, n: int) -> None:
        self.n = n
        self.bit = []
        i: int = 0
        while i <= n:
            self.bit.append(0)
            i = i + 1

    def add(self, i1: int, d: int) -> None:
        i: int = i1
        while i <= self.n:
            self.bit[i] = self.bit[i] + d
            i = i + (i & -i)

    def sum(self, i1: int) -> int:
        s: int = 0
        i: int = i1
        while i > 0:
            s = s + self.bit[i]
            i = i - (i & -i)
        return s

    def kth(self, k: int) -> int:
        idx: int = 0
        bit: int = 1
        while bit * 2 <= self.n:
            bit = bit * 2
        cur: int = 0
        while bit > 0:
            nxt: int = idx + bit
            if nxt <= self.n and cur + self.bit[nxt] < k:
                idx = nxt
                cur = cur + self.bit[nxt]
            bit = bit // 2
        return idx + 1


class OrderedMultiSet:
    lo: int
    hi: int
    fw: Fenwick
    freq: list[int]

    def __init__(self, lo: int, hi: int) -> None:
        self.lo = lo
        self.hi = hi
        n: int = hi - lo + 1
        self.fw = Fenwick(n)
        self.freq = []
        i: int = 0
        while i < n:
            self.freq.append(0)
            i = i + 1

    def _idx(self, x: int) -> int:
        return x - self.lo

    def add(self, x: int) -> None:
        id0: int = self._idx(x)
        self.freq[id0] = self.freq[id0] + 1
        self.fw.add(id0 + 1, 1)

    def remove_one(self, x: int) -> bool:
        id0: int = self._idx(x)
        if self.freq[id0] == 0:
            return False
        self.freq[id0] = self.freq[id0] - 1
        self.fw.add(id0 + 1, -1)
        return True

    def count(self, x: int) -> int:
        return self.freq[self._idx(x)]

    def size(self) -> int:
        return self.fw.sum(len(self.freq))

    def kth(self, k: int) -> int:
        return self.lo + self.fw.kth(k) - 1


class RollbackDSU:
    parent: list[int]
    size: list[int]
    hist_rb: list[int]
    hist_ra: list[int]
    hist_old_size_ra: list[int]
    comps: int

    def __init__(self, n: int) -> None:
        self.parent = []
        self.size = []
        i: int = 0
        while i < n:
            self.parent.append(i)
            self.size.append(1)
            i = i + 1
        self.hist_rb = []
        self.hist_ra = []
        self.hist_old_size_ra = []
        self.comps = n

    def find(self, x: int) -> int:
        while self.parent[x] != x:
            x = self.parent[x]
        return x

    def union(self, a: int, b: int) -> bool:
        ra: int = self.find(a)
        rb: int = self.find(b)
        if ra == rb:
            self.hist_rb.append(-1)
            self.hist_ra.append(-1)
            self.hist_old_size_ra.append(-1)
            return False
        if self.size[ra] < self.size[rb]:
            t: int = ra
            ra = rb
            rb = t
        self.hist_rb.append(rb)
        self.hist_ra.append(ra)
        self.hist_old_size_ra.append(self.size[ra])
        self.parent[rb] = ra
        self.size[ra] = self.size[ra] + self.size[rb]
        self.comps = self.comps - 1
        return True

    def snapshot(self) -> int:
        return len(self.hist_rb)

    def rollback(self, snap: int) -> None:
        while len(self.hist_rb) > snap:
            rb: int = self.hist_rb.pop()
            ra: int = self.hist_ra.pop()
            old_size_ra: int = self.hist_old_size_ra.pop()
            if rb == -1:
                continue
            self.parent[rb] = rb
            self.size[ra] = old_size_ra
            self.comps = self.comps + 1


def test_ordered_multiset_queries() -> None:
    ms: OrderedMultiSet = OrderedMultiSet(-2000, 2000)
    naive: list[int] = []
    i: int = 0
    while i < 4001:
        naive.append(0)
        i = i + 1

    t: int = 0
    while t < 12000:
        x: int = ((t * 89 + 17) % 4001) - 2000
        if t % 5 == 0:
            ms.remove_one(x)
            id0: int = x + 2000
            if naive[id0] > 0:
                naive[id0] = naive[id0] - 1
        else:
            ms.add(x)
            naive[x + 2000] = naive[x + 2000] + 1
        t = t + 1

    total_naive: int = 0
    j: int = 0
    while j < 4001:
        total_naive = total_naive + naive[j]
        j = j + 1
    print('CHECK test_ordered_multiset_rollback_dsu lhs:', ms.size())
    print('CHECK test_ordered_multiset_rollback_dsu rhs:', total_naive)
    assert ms.size() == total_naive

    if total_naive > 0:
        k: int = total_naive // 2 + 1
        kth_val: int = ms.kth(k)

        run: int = 0
        idx: int = 0
        while idx < 4001:
            run = run + naive[idx]
            if run >= k:
                break
            idx = idx + 1
        print('CHECK test_ordered_multiset_rollback_dsu lhs:', kth_val)
        print('CHECK test_ordered_multiset_rollback_dsu rhs:', idx - 2000)
        assert kth_val == idx - 2000
    print(total_naive)


def test_rollback_dsu_offline_style() -> None:
    n: int = 3000
    dsu: RollbackDSU = RollbackDSU(n)

    base_snap: int = dsu.snapshot()
    i: int = 1
    while i < n:
        dsu.union(i - 1, i)
        i = i + 1
    print('CHECK test_ordered_multiset_rollback_dsu lhs:', dsu.comps)
    print('CHECK test_ordered_multiset_rollback_dsu rhs:', 1)
    assert dsu.comps == 1

    dsu.rollback(base_snap)
    print('CHECK test_ordered_multiset_rollback_dsu lhs:', dsu.comps)
    print('CHECK test_ordered_multiset_rollback_dsu rhs:', n)
    assert dsu.comps == n

    s1: int = dsu.snapshot()
    j: int = 0
    while j + 2 < n:
        dsu.union(j, j + 2)
        j = j + 3
    mid_comps: int = dsu.comps
    print('CHECK test_ordered_multiset_rollback_dsu assert expr:', 'mid_comps < n')
    assert mid_comps < n

    s2: int = dsu.snapshot()
    k: int = 0
    while k + 1 < n:
        dsu.union(k, k + 1)
        k = k + 2
    print('CHECK test_ordered_multiset_rollback_dsu assert expr:', 'dsu.comps <= mid_comps')
    assert dsu.comps <= mid_comps

    dsu.rollback(s2)
    print('CHECK test_ordered_multiset_rollback_dsu lhs:', dsu.comps)
    print('CHECK test_ordered_multiset_rollback_dsu rhs:', mid_comps)
    assert dsu.comps == mid_comps
    dsu.rollback(s1)
    print('CHECK test_ordered_multiset_rollback_dsu lhs:', dsu.comps)
    print('CHECK test_ordered_multiset_rollback_dsu rhs:', n)
    assert dsu.comps == n
    print(mid_comps)


def run_tests() -> None:
    test_ordered_multiset_queries()
    test_rollback_dsu_offline_style()
