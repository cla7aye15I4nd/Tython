class MonotonicQueueMin:
    values: list[int]
    idxs: list[int]
    head: int

    def __init__(self) -> None:
        self.values = []
        self.idxs = []
        self.head = 0

    def push(self, idx: int, value: int) -> None:
        while len(self.values) > self.head and self.values[len(self.values) - 1] >= value:
            self.values.pop()
            self.idxs.pop()
        self.values.append(value)
        self.idxs.append(idx)

    def pop_expired(self, min_idx: int) -> None:
        while self.head < len(self.idxs) and self.idxs[self.head] < min_idx:
            self.head = self.head + 1

    def get_min(self) -> int:
        return self.values[self.head]


class MonotonicStackNextGreater:
    st_idx: list[int]
    st_val: list[int]

    def __init__(self) -> None:
        self.st_idx = []
        self.st_val = []

    def process(self, arr: list[int]) -> list[int]:
        res: list[int] = []
        i: int = 0
        while i < len(arr):
            res.append(-1)
            i = i + 1

        j: int = 0
        while j < len(arr):
            v: int = arr[j]
            while len(self.st_val) > 0 and self.st_val[len(self.st_val) - 1] < v:
                idx: int = self.st_idx.pop()
                self.st_val.pop()
                res[idx] = j
            self.st_idx.append(j)
            self.st_val.append(v)
            j = j + 1
        return res


def test_sliding_window_minimum_large() -> None:
    n: int = 12000
    w: int = 53
    arr: list[int] = []
    i: int = 0
    while i < n:
        arr.append(((i * 97 + 41) % 100003) - 50000)
        i = i + 1

    mq: MonotonicQueueMin = MonotonicQueueMin()
    mins: list[int] = []
    j: int = 0
    while j < n:
        mq.push(j, arr[j])
        if j + 1 >= w:
            mq.pop_expired(j - w + 1)
            mins.append(mq.get_min())
        j = j + 1

    assert len(mins) == n - w + 1

    q: int = 0
    checksum: int = 0
    while q < len(mins):
        naive: int = arr[q]
        k: int = q + 1
        while k < q + w:
            if arr[k] < naive:
                naive = arr[k]
            k = k + 1
        assert mins[q] == naive
        checksum = (checksum + (naive % 1000000007 + 1000000007) % 1000000007) % 1000000007
        q = q + 97

    print(checksum)


def test_next_greater_indices_large() -> None:
    n: int = 10000
    arr: list[int] = []
    i: int = 0
    while i < n:
        arr.append((i * 67 + 23) % 1000)
        i = i + 1

    ms: MonotonicStackNextGreater = MonotonicStackNextGreater()
    nxt: list[int] = ms.process(arr)
    assert len(nxt) == n

    q: int = 0
    verified: int = 0
    while q < n:
        got: int = nxt[q]
        if got != -1:
            assert got > q
            assert arr[got] > arr[q]
            t: int = q + 1
            while t < got:
                assert arr[t] <= arr[q]
                t = t + 1
            verified = verified + 1
        q = q + 1

    assert verified > 0
    print(verified)


def run_tests() -> None:
    test_sliding_window_minimum_large()
    test_next_greater_indices_large()
