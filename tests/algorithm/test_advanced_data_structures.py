class LRUCache:
    capacity: int
    keys: list[int]
    values: list[int]

    def __init__(self, capacity: int) -> None:
        self.capacity = capacity
        self.keys = []
        self.values = []

    def _find(self, key: int) -> int:
        i: int = 0
        while i < len(self.keys):
            if self.keys[i] == key:
                return i
            i = i + 1
        return -1

    def get(self, key: int) -> int:
        idx: int = self._find(key)
        if idx == -1:
            return -1

        val: int = self.values[idx]
        # Move key/value to MRU position (end), in place.
        i: int = idx
        while i + 1 < len(self.keys):
            self.keys[i] = self.keys[i + 1]
            self.values[i] = self.values[i + 1]
            i = i + 1
        self.keys[len(self.keys) - 1] = key
        self.values[len(self.values) - 1] = val
        return val

    def put(self, key: int, value: int) -> None:
        idx: int = self._find(key)

        if idx != -1:
            # Update existing and move to MRU.
            self.values[idx] = value
            _ = self.get(key)
            return

        if len(self.keys) == self.capacity:
            # Evict LRU (front).
            i: int = 1
            while i < len(self.keys):
                self.keys[i - 1] = self.keys[i]
                self.values[i - 1] = self.values[i]
                i = i + 1
            self.keys.pop()
            self.values.pop()

        self.keys.append(key)
        self.values.append(value)


class DigitTrie:
    children: list[int]
    terminal: list[bool]

    def __init__(self) -> None:
        self.children = []
        self.terminal = []
        self._new_node()

    def _new_node(self) -> int:
        node_id: int = len(self.terminal)
        i: int = 0
        while i < 10:
            self.children.append(-1)
            i = i + 1
        self.terminal.append(False)
        return node_id

    def insert(self, digits: list[int]) -> None:
        node: int = 0
        i: int = 0
        while i < len(digits):
            d: int = digits[i]
            slot: int = node * 10 + d
            nxt: int = self.children[slot]
            if nxt == -1:
                nxt = self._new_node()
                self.children[slot] = nxt
            node = nxt
            i = i + 1
        self.terminal[node] = True

    def contains(self, digits: list[int]) -> bool:
        node: int = 0
        i: int = 0
        while i < len(digits):
            d: int = digits[i]
            nxt: int = self.children[node * 10 + d]
            if nxt == -1:
                return False
            node = nxt
            i = i + 1
        return self.terminal[node]

    def has_prefix(self, digits: list[int]) -> bool:
        node: int = 0
        i: int = 0
        while i < len(digits):
            d: int = digits[i]
            nxt: int = self.children[node * 10 + d]
            if nxt == -1:
                return False
            node = nxt
            i = i + 1
        return True


class OrderStatisticFenwick:
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
        s: int = 0
        i: int = idx_1based
        while i > 0:
            s = s + self.bit[i]
            i = i - (i & -i)
        return s

    def kth(self, k: int) -> int:
        # Smallest idx such that prefix_sum(idx) >= k, assuming 1 <= k <= total.
        idx: int = 0
        step: int = 1
        while step * 2 <= self.n:
            step = step * 2

        cur: int = 0
        while step > 0:
            nxt: int = idx + step
            if nxt <= self.n and cur + self.bit[nxt] < k:
                idx = nxt
                cur = cur + self.bit[nxt]
            step = step // 2
        return idx + 1


def to_digits_6(x: int) -> list[int]:
    d: list[int] = [0, 0, 0, 0, 0, 0]
    v: int = x
    i: int = 5
    while i >= 0:
        d[i] = v % 10
        v = v // 10
        i = i - 1
    return d


def lru_ref_find(rk: list[int], key: int) -> int:
    i: int = 0
    while i < len(rk):
        if rk[i] == key:
            return i
        i = i + 1
    return -1


def has_prefix_in_values(inserted: list[int], pref: list[int]) -> bool:
    i: int = 0
    while i < len(inserted):
        d6: list[int] = to_digits_6(inserted[i])
        ok: bool = True
        j: int = 0
        while j < len(pref):
            if d6[j] != pref[j]:
                ok = False
                break
            j = j + 1
        if ok:
            return True
        i = i + 1
    return False


def test_lru_cache_heavy_sequence() -> None:
    cap: int = 64
    cache: LRUCache = LRUCache(cap)

    # Reference model with same semantics (simple lists).
    rk: list[int] = []
    rv: list[int] = []

    op: int = 0
    checksum: int = 0
    while op < 6000:
        key: int = (op * 37 + 11) % 180
        if op % 5 == 0:
            got: int = cache.get(key)

            ridx: int = lru_ref_find(rk, key)
            expected: int = -1
            if ridx != -1:
                expected = rv[ridx]
                nk: list[int] = []
                nv: list[int] = []
                j: int = 0
                while j < len(rk):
                    if j != ridx:
                        nk.append(rk[j])
                        nv.append(rv[j])
                    j = j + 1
                nk.append(key)
                nv.append(expected)
                rk = nk
                rv = nv

            print('CHECK test_advanced_data_structures lhs:', got)
            print('CHECK test_advanced_data_structures rhs:', expected)
            assert got == expected
            checksum = (checksum + (got + 1000) * (op + 1)) % 1000000007
        else:
            value: int = ((op * 91 + 19) % 100000) - 50000
            cache.put(key, value)

            ridx2: int = lru_ref_find(rk, key)
            if ridx2 != -1:
                rv[ridx2] = value
                nk2: list[int] = []
                nv2: list[int] = []
                j2: int = 0
                while j2 < len(rk):
                    if j2 != ridx2:
                        nk2.append(rk[j2])
                        nv2.append(rv[j2])
                    j2 = j2 + 1
                nk2.append(key)
                nv2.append(value)
                rk = nk2
                rv = nv2
            else:
                if len(rk) == cap:
                    tk: list[int] = []
                    tv: list[int] = []
                    p: int = 1
                    while p < len(rk):
                        tk.append(rk[p])
                        tv.append(rv[p])
                        p = p + 1
                    rk = tk
                    rv = tv
                rk.append(key)
                rv.append(value)

        op = op + 1

    # Full keyspace validation.
    x: int = 0
    while x < 180:
        got2: int = cache.get(x)
        idx3: int = lru_ref_find(rk, x)
        if idx3 == -1:
            print('CHECK test_advanced_data_structures lhs:', got2)
            print('CHECK test_advanced_data_structures rhs:', -1)
            assert got2 == -1
        else:
            print('CHECK test_advanced_data_structures lhs:', got2)
            print('CHECK test_advanced_data_structures rhs:', rv[idx3])
            assert got2 == rv[idx3]
        x = x + 1

    print(checksum)


def test_digit_trie_bulk_operations() -> None:
    trie: DigitTrie = DigitTrie()
    present: list[bool] = []
    inserted: list[int] = []
    i: int = 0
    while i < 1000000:
        present.append(False)
        i = i + 1

    k: int = 0
    while k < 2500:
        val: int = (k * 92821 + 12345) % 1000000
        trie.insert(to_digits_6(val))
        inserted.append(val)
        present[val] = True
        k = k + 1

    checks: int = 0
    q: int = 0
    while q < 4000:
        probe: int = (q * 73129 + 77) % 1000000
        got: bool = trie.contains(to_digits_6(probe))
        print('CHECK test_advanced_data_structures lhs:', got)
        print('CHECK test_advanced_data_structures rhs:', present[probe])
        assert got == present[probe]
        if got:
            checks = checks + 1
        q = q + 1

    p2: int = 0
    while p2 < 1000:
        base: int = (p2 * 1000 + 507) % 1000000
        d6: list[int] = to_digits_6(base)
        pref3: list[int] = [d6[0], d6[1], d6[2]]
        print('CHECK test_advanced_data_structures lhs expr:', 'trie.has_prefix(pref3)')
        print('CHECK test_advanced_data_structures rhs expr:', 'has_prefix_in_values(inserted, pref3)')
        assert trie.has_prefix(pref3) == has_prefix_in_values(inserted, pref3)
        p2 = p2 + 1

    print(checks)


def test_order_statistic_fenwick_kth() -> None:
    n: int = 4096
    fw: OrderStatisticFenwick = OrderStatisticFenwick(n)
    freq: list[int] = []
    i: int = 0
    while i < n:
        freq.append(0)
        i = i + 1

    step: int = 0
    while step < 20000:
        idx0: int = (step * 97 + 31) % n
        delta: int = 1
        if step % 4 == 0:
            delta = 1
        else:
            if freq[idx0] > 0 and step % 7 == 0:
                delta = -1
            else:
                delta = 1

        freq[idx0] = freq[idx0] + delta
        fw.add(idx0 + 1, delta)
        step = step + 1

    total: int = fw.prefix_sum(n)
    print('CHECK test_advanced_data_structures assert expr:', 'total > 0')
    assert total > 0

    q: int = 1
    checksum2: int = 0
    while q <= 2000:
        k: int = (q * 37 + 11) % total + 1
        idx1: int = fw.kth(k)

        running: int = 0
        naive_idx: int = 1
        while naive_idx <= n:
            running = running + freq[naive_idx - 1]
            if running >= k:
                break
            naive_idx = naive_idx + 1

        print('CHECK test_advanced_data_structures lhs:', idx1)
        print('CHECK test_advanced_data_structures rhs:', naive_idx)
        assert idx1 == naive_idx
        checksum2 = (checksum2 + idx1 * q) % 1000000007
        q = q + 1

    print(checksum2)


def run_tests() -> None:
    test_lru_cache_heavy_sequence()
    test_digit_trie_bulk_operations()
    test_order_statistic_fenwick_kth()
