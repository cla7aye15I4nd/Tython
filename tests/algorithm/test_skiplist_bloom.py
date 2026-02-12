class SkipList:
    key: list[int]
    nxt: list[int]
    level: list[int]
    head: int
    max_level: int

    def __init__(self, max_level: int) -> None:
        self.key = []
        self.nxt = []
        self.level = []
        self.max_level = max_level
        self.head = self._new_node(-2147483648, max_level)

    def _next_idx(self, node: int, lv: int) -> int:
        return node * self.max_level + lv

    def _new_next_row(self) -> None:
        i: int = 0
        while i < self.max_level:
            self.nxt.append(-1)
            i = i + 1

    def _new_node(self, k: int, lvl: int) -> int:
        self.key.append(k)
        self._new_next_row()
        self.level.append(lvl)
        return len(self.key) - 1

    def _level_for(self, k: int) -> int:
        h: int = (k * 1103515245 + 12345) & 2147483647
        lvl: int = 1
        while lvl < self.max_level and h % 4 == 0:
            lvl = lvl + 1
            h = h // 4
        return lvl

    def add(self, k: int) -> None:
        update: list[int] = []
        i: int = 0
        while i < self.max_level:
            update.append(self.head)
            i = i + 1

        cur: int = self.head
        lv: int = self.max_level - 1
        while lv >= 0:
            while self.nxt[self._next_idx(cur, lv)] != -1 and self.key[self.nxt[self._next_idx(cur, lv)]] < k:
                cur = self.nxt[self._next_idx(cur, lv)]
            update[lv] = cur
            lv = lv - 1

        c0: int = self.nxt[self._next_idx(update[0], 0)]
        if c0 != -1 and self.key[c0] == k:
            return

        node_lvl: int = self._level_for(k)
        idx: int = self._new_node(k, node_lvl)
        j: int = 0
        while j < node_lvl:
            self.nxt[self._next_idx(idx, j)] = self.nxt[self._next_idx(update[j], j)]
            self.nxt[self._next_idx(update[j], j)] = idx
            j = j + 1

    def contains(self, k: int) -> bool:
        cur: int = self.head
        lv: int = self.max_level - 1
        while lv >= 0:
            while self.nxt[self._next_idx(cur, lv)] != -1 and self.key[self.nxt[self._next_idx(cur, lv)]] < k:
                cur = self.nxt[self._next_idx(cur, lv)]
            lv = lv - 1
        cur = self.nxt[self._next_idx(cur, 0)]
        return cur != -1 and self.key[cur] == k


class BloomFilter:
    m: int
    bits: list[bool]

    def __init__(self, m: int) -> None:
        self.m = m
        self.bits = []
        i: int = 0
        while i < m:
            self.bits.append(False)
            i = i + 1

    def _h1(self, x: int) -> int:
        v: int = (x * 73856093 + 19349663) & 2147483647
        return v % self.m

    def _h2(self, x: int) -> int:
        v: int = (x * 83492791 + 12345) & 2147483647
        return v % self.m

    def _h3(self, x: int) -> int:
        v: int = (x * 2654435761 + 10007) & 2147483647
        return v % self.m

    def add(self, x: int) -> None:
        self.bits[self._h1(x)] = True
        self.bits[self._h2(x)] = True
        self.bits[self._h3(x)] = True

    def maybe_contains(self, x: int) -> bool:
        return self.bits[self._h1(x)] and self.bits[self._h2(x)] and self.bits[self._h3(x)]


def test_skiplist_large_membership() -> None:
    sl: SkipList = SkipList(12)
    used: list[bool] = []
    i: int = 0
    while i < 20000:
        used.append(False)
        i = i + 1

    j: int = 0
    while j < 7000:
        v: int = (j * 113 + 29) % 20000
        sl.add(v)
        used[v] = True
        j = j + 1

    k: int = 0
    while k < 20000:
        print('CHECK test_skiplist_bloom lhs expr:', 'sl.contains(k)')
        print('CHECK test_skiplist_bloom rhs:', used[k])
        assert sl.contains(k) == used[k]
        k = k + 1
    print(7000)


def test_bloom_filter_false_positive_rate() -> None:
    m: int = 50000
    bf: BloomFilter = BloomFilter(m)
    inserted: list[bool] = []
    i: int = 0
    while i < 120000:
        inserted.append(False)
        i = i + 1

    j: int = 0
    while j < 8000:
        x: int = (j * 97 + 43) % 120000
        inserted[x] = True
        bf.add(x)
        j = j + 1

    false_pos: int = 0
    negatives: int = 0
    q: int = 0
    while q < 50000:
        y: int = (q * 131 + 17) % 120000
        if not inserted[y]:
            negatives = negatives + 1
            if bf.maybe_contains(y):
                false_pos = false_pos + 1
        else:
            print('CHECK test_skiplist_bloom assert expr:', 'bf.maybe_contains(y)')
            assert bf.maybe_contains(y)
        q = q + 1

    # Loose upper bound to keep test stable.
    print('CHECK test_skiplist_bloom assert expr:', 'false_pos * 100 <= negatives * 30')
    assert false_pos * 100 <= negatives * 30
    print(false_pos)


def run_tests() -> None:
    test_skiplist_large_membership()
    test_bloom_filter_false_positive_rate()
