class IntervalTree:
    lo: list[int]
    hi: list[int]
    mx: list[int]
    left: list[int]
    right: list[int]
    root: int

    def __init__(self) -> None:
        self.lo = []
        self.hi = []
        self.mx = []
        self.left = []
        self.right = []
        self.root = -1

    def _new_node(self, l: int, r: int) -> int:
        self.lo.append(l)
        self.hi.append(r)
        self.mx.append(r)
        self.left.append(-1)
        self.right.append(-1)
        return len(self.lo) - 1

    def _insert_node(self, n: int, l: int, r: int) -> int:
        if n == -1:
            return self._new_node(l, r)
        if l < self.lo[n]:
            self.left[n] = self._insert_node(self.left[n], l, r)
        else:
            self.right[n] = self._insert_node(self.right[n], l, r)
        if self.mx[n] < r:
            self.mx[n] = r
        return n

    def add(self, l: int, r: int) -> None:
        self.root = self._insert_node(self.root, l, r)

    def any_overlap(self, ql: int, qr: int) -> bool:
        st: list[int] = []
        if self.root != -1:
            st.append(self.root)
        while len(st) > 0:
            n: int = st.pop()
            if not (self.hi[n] < ql or self.lo[n] > qr):
                return True
            if self.left[n] != -1 and self.mx[self.left[n]] >= ql:
                st.append(self.left[n])
            if self.right[n] != -1 and self.lo[n] <= qr:
                st.append(self.right[n])
        return False


class Fenwick2D:
    n: int
    m: int
    bit: list[int]

    def __init__(self, n: int, m: int) -> None:
        self.n = n
        self.m = m
        self.bit = []
        i: int = 0
        total: int = (n + 1) * (m + 1)
        while i < total:
            self.bit.append(0)
            i = i + 1

    def _id(self, x: int, y: int) -> int:
        return x * (self.m + 1) + y

    def add(self, x1: int, y1: int, delta: int) -> None:
        i: int = x1
        while i <= self.n:
            j: int = y1
            while j <= self.m:
                idx: int = self._id(i, j)
                self.bit[idx] = self.bit[idx] + delta
                j = j + (j & -j)
            i = i + (i & -i)

    def sum(self, x1: int, y1: int) -> int:
        res: int = 0
        i: int = x1
        while i > 0:
            j: int = y1
            while j > 0:
                res = res + self.bit[self._id(i, j)]
                j = j - (j & -j)
            i = i - (i & -i)
        return res

    def rect_sum(self, x1: int, y1: int, x2: int, y2: int) -> int:
        a: int = self.sum(x2, y2)
        b: int = self.sum(x1 - 1, y2)
        c: int = self.sum(x2, y1 - 1)
        d: int = self.sum(x1 - 1, y1 - 1)
        return a - b - c + d


def test_interval_tree_overlap_queries() -> None:
    it: IntervalTree = IntervalTree()
    starts: list[int] = []
    ends: list[int] = []
    i: int = 0
    while i < 6000:
        l: int = (i * 37 + 11) % 20000
        w: int = (i * 13 + 3) % 30 + 1
        r: int = l + w
        it.add(l, r)
        starts.append(l)
        ends.append(r)
        i = i + 1

    q: int = 0
    hits: int = 0
    while q < 3000:
        ql: int = (q * 71 + 19) % 20000
        qr: int = ql + ((q * 29 + 5) % 40)

        naive: bool = False
        j: int = 0
        while j < len(starts):
            if not (ends[j] < ql or starts[j] > qr):
                naive = True
                break
            j = j + 1

        got: bool = it.any_overlap(ql, qr)
        assert got == naive
        if got:
            hits = hits + 1
        q = q + 1
    print(hits)


def test_fenwick_2d_updates_queries() -> None:
    n: int = 64
    m: int = 64
    fw: Fenwick2D = Fenwick2D(n, m)
    grid: list[int] = []

    i: int = 0
    while i < n * m:
        grid.append(0)
        i = i + 1

    k: int = 0
    while k < 8000:
        x: int = (k * 17 + 7) % n
        y: int = (k * 23 + 13) % m
        d: int = ((k * 31 + 9) % 41) - 20
        gid: int = x * m + y
        grid[gid] = grid[gid] + d
        fw.add(x + 1, y + 1, d)
        k = k + 1

    q: int = 0
    checksum: int = 0
    while q < 2000:
        x1: int = (q * 11 + 3) % n
        y1: int = (q * 19 + 5) % m
        x2: int = x1 + ((q * 7 + 1) % 12)
        y2: int = y1 + ((q * 13 + 2) % 12)
        if x2 >= n:
            x2 = n - 1
        if y2 >= m:
            y2 = m - 1

        naive: int = 0
        a: int = x1
        while a <= x2:
            b: int = y1
            while b <= y2:
                naive = naive + grid[a * m + b]
                b = b + 1
            a = a + 1

        got: int = fw.rect_sum(x1 + 1, y1 + 1, x2 + 1, y2 + 1)
        assert got == naive
        checksum = (checksum + (got % 1000000007 + 1000000007) % 1000000007) % 1000000007
        q = q + 1
    print(checksum)


def run_tests() -> None:
    test_interval_tree_overlap_queries()
    test_fenwick_2d_updates_queries()
