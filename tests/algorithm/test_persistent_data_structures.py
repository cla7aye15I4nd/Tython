class PersistentStack:
    value: list[int]
    prev: list[int]
    head_versions: list[int]

    def __init__(self) -> None:
        self.value = []
        self.prev = []
        self.head_versions = [-1]

    def push(self, ver: int, x: int) -> int:
        head: int = self.head_versions[ver]
        self.value.append(x)
        self.prev.append(head)
        new_head: int = len(self.value) - 1
        self.head_versions.append(new_head)
        return len(self.head_versions) - 1

    def pop(self, ver: int) -> tuple[int, int]:
        head: int = self.head_versions[ver]
        if head == -1:
            self.head_versions.append(-1)
            return (len(self.head_versions) - 1, -1)
        x: int = self.value[head]
        new_head: int = self.prev[head]
        self.head_versions.append(new_head)
        return (len(self.head_versions) - 1, x)

    def top(self, ver: int) -> int:
        head: int = self.head_versions[ver]
        if head == -1:
            return -1
        return self.value[head]


class PersistentArray:
    n: int
    versions: list[int]
    version_count: int

    def __init__(self, n: int) -> None:
        self.n = n
        self.versions = []
        i: int = 0
        while i < n:
            self.versions.append(0)
            i = i + 1
        self.version_count = 1

    def set_value(self, ver: int, idx: int, value: int) -> int:
        old_base: int = ver * self.n
        i: int = 0
        while i < self.n:
            if i == idx:
                self.versions.append(value)
            else:
                self.versions.append(self.versions[old_base + i])
            i = i + 1
        self.version_count = self.version_count + 1
        return self.version_count - 1

    def get(self, ver: int, idx: int) -> int:
        return self.versions[ver * self.n + idx]

    def range_sum(self, ver: int, l: int, r: int) -> int:
        s: int = 0
        base: int = ver * self.n
        i: int = l
        while i <= r:
            s = s + self.versions[base + i]
            i = i + 1
        return s


def test_persistent_stack_version_branching() -> None:
    ps: PersistentStack = PersistentStack()

    v1: int = ps.push(0, 10)
    v2: int = ps.push(v1, 20)
    v3: int = ps.push(v2, 30)
    assert ps.top(v1) == 10
    assert ps.top(v2) == 20
    assert ps.top(v3) == 30

    p1: tuple[int, int] = ps.pop(v3)
    v4: int = p1[0]
    x1: int = p1[1]
    assert x1 == 30
    assert ps.top(v4) == 20
    assert ps.top(v3) == 30

    v5: int = ps.push(v1, 99)
    assert ps.top(v5) == 99
    assert ps.top(v2) == 20

    p2: tuple[int, int] = ps.pop(v5)
    assert p2[1] == 99
    assert ps.top(p2[0]) == 10
    print(ps.top(v2))


def test_persistent_array_many_versions() -> None:
    n: int = 256
    pa: PersistentArray = PersistentArray(n)
    vers: list[int] = [0]

    i: int = 0
    while i < 2000:
        base_ver: int = vers[(i * 17 + 3) % len(vers)]
        idx: int = (i * 29 + 11) % n
        val: int = ((i * 97 + 19) % 10000) - 5000
        new_ver: int = pa.set_value(base_ver, idx, val)
        vers.append(new_ver)
        i = i + 1

    q: int = 0
    checksum: int = 0
    while q < 1000:
        ver: int = vers[(q * 31 + 7) % len(vers)]
        l: int = (q * 13 + 5) % n
        r: int = l + ((q * 19 + 1) % 20)
        if r >= n:
            r = n - 1

        s: int = pa.range_sum(ver, l, r)
        checksum = (checksum + (s % 1000000007 + 1000000007) % 1000000007) % 1000000007
        q = q + 1

    # Persistence check: older versions remain unchanged.
    assert pa.get(0, 0) == 0
    assert pa.get(0, n - 1) == 0
    print(checksum)


def run_tests() -> None:
    test_persistent_stack_version_branching()
    test_persistent_array_many_versions()
