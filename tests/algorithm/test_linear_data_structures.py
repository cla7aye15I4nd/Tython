class IntStack:
    data: list[int]

    def __init__(self) -> None:
        self.data = []

    def push(self, x: int) -> None:
        self.data.append(x)

    def pop(self) -> int:
        return self.data.pop()

    def top(self) -> int:
        return self.data[len(self.data) - 1]

    def __len__(self) -> int:
        return len(self.data)


class IntQueue:
    data: list[int]
    head: int

    def __init__(self) -> None:
        self.data = []
        self.head = 0

    def push(self, x: int) -> None:
        self.data.append(x)

    def pop(self) -> int:
        x: int = self.data[self.head]
        self.head = self.head + 1
        if self.head * 2 >= len(self.data):
            write: int = 0
            i: int = self.head
            while i < len(self.data):
                self.data[write] = self.data[i]
                write = write + 1
                i = i + 1
            while len(self.data) > write:
                self.data.pop()
            self.head = 0
        return x

    def __len__(self) -> int:
        return len(self.data) - self.head


class IntDeque:
    buf: list[int]
    head: int
    size: int

    def __init__(self) -> None:
        self.buf = [0, 0, 0, 0]
        self.head = 0
        self.size = 0

    def _grow(self) -> None:
        old_cap: int = len(self.buf)
        i: int = 0
        while i < old_cap:
            self.buf.append(0)
            i = i + 1

        wrap: int = self.head + self.size - old_cap
        if wrap > 0:
            j: int = 0
            while j < wrap:
                self.buf[old_cap + j] = self.buf[j]
                j = j + 1

    def push_back(self, x: int) -> None:
        if self.size == len(self.buf):
            self._grow()
        idx: int = (self.head + self.size) % len(self.buf)
        self.buf[idx] = x
        self.size = self.size + 1

    def push_front(self, x: int) -> None:
        if self.size == len(self.buf):
            self._grow()
        self.head = (self.head - 1 + len(self.buf)) % len(self.buf)
        self.buf[self.head] = x
        self.size = self.size + 1

    def pop_front(self) -> int:
        x: int = self.buf[self.head]
        self.head = (self.head + 1) % len(self.buf)
        self.size = self.size - 1
        return x

    def pop_back(self) -> int:
        idx: int = (self.head + self.size - 1) % len(self.buf)
        x: int = self.buf[idx]
        self.size = self.size - 1
        return x

    def __len__(self) -> int:
        return self.size


class IntHashSet:
    keys: list[int]
    used: list[bool]
    count: int

    def __init__(self) -> None:
        self.keys = []
        self.used = []
        self.count = 0

        i: int = 0
        while i < 16384:
            self.keys.append(0)
            self.used.append(False)
            i = i + 1

    def _mix(self, x: int) -> int:
        y: int = x * 1103515245 + 12345
        if y < 0:
            y = -y
        return y

    def _rehash(self) -> None:
        old_keys: list[int] = []
        old_used: list[bool] = []
        i: int = 0
        while i < len(self.keys):
            old_keys.append(self.keys[i])
            old_used.append(self.used[i])
            i = i + 1

        old_cap: int = len(self.keys)
        while len(self.keys) < old_cap * 2:
            self.keys.append(0)
            self.used.append(False)
        i = 0
        while i < len(self.keys):
            self.keys[i] = 0
            self.used[i] = False
            i = i + 1
        self.count = 0

        j: int = 0
        while j < old_cap:
            if old_used[j]:
                self.add(old_keys[j])
            j = j + 1

    def add(self, x: int) -> bool:
        cap: int = len(self.keys)
        idx: int = self._mix(x) % cap
        while self.used[idx]:
            if self.keys[idx] == x:
                return False
            idx = (idx + 1) % cap

        self.used[idx] = True
        self.keys[idx] = x
        self.count = self.count + 1
        return True

    def contains(self, x: int) -> bool:
        cap: int = len(self.keys)
        idx: int = self._mix(x) % cap
        while self.used[idx]:
            if self.keys[idx] == x:
                return True
            idx = (idx + 1) % cap
        return False

    def __len__(self) -> int:
        return self.count


def test_stack_and_queue_stress() -> None:
    n: int = 12000
    st: IntStack = IntStack()
    q: IntQueue = IntQueue()

    i: int = 0
    while i < n:
        v: int = (i * 31 + 9) % 100003
        st.push(v)
        q.push(v)
        i = i + 1

    checksum_stack: int = 0
    j: int = n - 1
    while j >= 0:
        x: int = st.pop()
        expected: int = (j * 31 + 9) % 100003
        assert x == expected
        checksum_stack = (checksum_stack + x * (j + 1)) % 1000000007
        j = j - 1
    assert len(st) == 0

    checksum_queue: int = 0
    k: int = 0
    while k < n:
        y: int = q.pop()
        expected_q: int = (k * 31 + 9) % 100003
        assert y == expected_q
        checksum_queue = (checksum_queue + y * (k + 1)) % 1000000007
        k = k + 1
    assert len(q) == 0

    assert checksum_stack == checksum_queue
    print(checksum_queue)


def test_deque_bidirectional_ops() -> None:
    dq: IntDeque = IntDeque()

    i: int = 0
    while i < 6000:
        if i % 2 == 0:
            dq.push_front(i)
        else:
            dq.push_back(i)
        i = i + 1

    assert len(dq) == 6000

    checksum: int = 0
    turn: int = 0
    expected_front: int = 5998
    expected_back: int = 5999
    while len(dq) > 0:
        if turn % 2 == 0:
            a: int = dq.pop_front()
            assert a == expected_front
            expected_front = expected_front - 2
        else:
            c: int = dq.pop_back()
            assert c == expected_back
            expected_back = expected_back - 2
        checksum = (checksum + (turn + 1) * (turn % 100 + 1)) % 1000000007
        turn = turn + 1

    assert expected_front == -2
    assert expected_back == -1
    print(checksum)


def test_hash_set_large_unique_tracking() -> None:
    hs: IntHashSet = IntHashSet()
    universe: int = 5000
    seen: list[bool] = []

    i: int = 0
    while i < universe:
        seen.append(False)
        i = i + 1

    inserts: int = 15000
    j: int = 0
    while j < inserts:
        v: int = ((j * 97 + 53) % universe) - 2000
        normalized: int = (v + 2000) % universe
        hs.add(v)
        seen[normalized] = True
        j = j + 1

    expected_unique: int = 0
    k: int = 0
    while k < universe:
        if seen[k]:
            expected_unique = expected_unique + 1
        k = k + 1

    assert len(hs) == expected_unique

    t: int = 0
    while t < universe:
        value: int = t - 2000
        assert hs.contains(value) == seen[t]
        t = t + 1

    print(len(hs))


def run_tests() -> None:
    test_stack_and_queue_stress()
    test_deque_bidirectional_ops()
    test_hash_set_large_unique_tracking()
