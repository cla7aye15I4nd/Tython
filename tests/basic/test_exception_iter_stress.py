class PairWalk:
    i: int
    j: int
    max_i: int
    max_j: int

    def __init__(self, max_i: int, max_j: int) -> None:
        self.i = 1
        self.j = 1
        self.max_i = max_i
        self.max_j = max_j

    def __iter__(self) -> "PairWalk":
        return self

    def __next__(self) -> tuple[int, int]:
        if self.i > self.max_i:
            raise StopIteration()

        out: tuple[int, int] = (self.i, self.j)
        self.j = self.j + 1
        if self.j > self.max_j:
            self.j = 1
            self.i = self.i + 1
        return out


def guarded_value(n: int, events: list[int]) -> int:
    result = 0
    try:
        events.append(1)
        if n == 0:
            raise Exception("zero")
        if n < 0:
            raise Exception("neg")
        result = n * 2
    except Exception:
        events.append(2)
        result = -10
    finally:
        events.append(3)
    return result


def test_try_except_finally_ordering_stress() -> None:
    events: list[int] = []
    values: list[int] = []

    for n in [3, 0, -1, 2]:
        try:
            v: int = guarded_value(n, events)
            if v < 0:
                raise Exception("bad value")
            values.append(v)
        except Exception:
            values.append(99)
        finally:
            values.append(7)

    assert values == [6, 7, 99, 7, 99, 7, 4, 7]
    assert events == [1, 3, 1, 2, 3, 1, 2, 3, 1, 3]


def test_manual_next_loop_and_repeated_stopiteration() -> None:
    it: PairWalk = PairWalk(2, 3)
    seen: list[int] = []
    stop_count: int = 0

    while True:
        try:
            pair: tuple[int, int] = it.__next__()
            seen.append(pair[0] * 10 + pair[1])
        except StopIteration:
            stop_count = stop_count + 1
            break

    try:
        it.__next__()
    except StopIteration:
        stop_count = stop_count + 1

    assert seen == [11, 12, 13, 21, 22, 23]
    assert stop_count == 2


def test_for_iterable_nested_control_flow() -> None:
    total: int = 0
    trace: list[int] = []

    for pair in PairWalk(3, 3):
        i: int = pair[0]
        j: int = pair[1]

        if i == 2 and j == 1:
            continue
        if i == 3 and j == 2:
            break

        try:
            if (i * j) % 5 == 0:
                raise Exception("skip")
            total = total + i * j
            trace.append(i * 10 + j)
        except Exception:
            total = total + 100
            trace.append(-1)
        finally:
            trace.append(0)

    assert total == 19
    assert trace == [11, 0, 12, 0, 13, 0, 22, 0, 23, 0, 31, 0]


def test_comprehensions_with_pairs_and_filters() -> None:
    products: list[int] = [p[0] * p[1] for p in PairWalk(4, 3)]
    assert products == [1, 2, 3, 2, 4, 6, 3, 6, 9, 4, 8, 12]

    even_pairs_code: list[int] = [
        p[0] * 10 + p[1] for p in PairWalk(4, 4) if (p[0] + p[1]) % 2 == 0
    ]
    assert even_pairs_code == [11, 13, 22, 24, 31, 33, 42, 44]

    layered_code: list[int] = [
        i * 100 + j * 10 + i * j for i in range(1, 5) for j in range(1, 5) if (i + j) % 2 == 1
    ]
    assert layered_code == [122, 144, 212, 236, 326, 352, 414, 442]

    tuple_summary: tuple[tuple[int, int], tuple[int, int], tuple[int, int], tuple[int, int]] = (
        tuple((i, 2) for i in range(1, 5))
    )
    assert tuple_summary == ((1, 2), (2, 2), (3, 2), (4, 2))


def run_tests() -> None:
    test_try_except_finally_ordering_stress()
    test_manual_next_loop_and_repeated_stopiteration()
    test_for_iterable_nested_control_flow()
    test_comprehensions_with_pairs_and_filters()
