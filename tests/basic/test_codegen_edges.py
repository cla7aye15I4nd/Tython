class TinyIter:
    current: int
    stop: int

    def __init__(self, stop: int) -> None:
        self.current = 0
        self.stop = stop

    def __iter__(self) -> "TinyIter":
        return self

    def __next__(self) -> int:
        if self.current >= self.stop:
            raise StopIteration()
        self.current = self.current + 1
        return self.current


def sink(n: int) -> None:
    _x: int = n + 1


def test_indirect_void_call() -> None:
    fn: "callable[[int], None]" = sink
    fn(41)


def test_for_range_else() -> None:
    total: int = 0
    for i in range(1, 4):
        total = total + i
    else:
        total = total + 10
    assert total == 16


def test_try_finally_without_except() -> None:
    events: list[int] = []
    try:
        events.append(1)
    finally:
        events.append(2)
    assert events == [1, 2]


def classify_exception(flag: int) -> str:
    try:
        if flag == 0:
            raise ValueError("value")
        if flag == 1:
            raise TypeError("type")
        if flag == 2:
            raise RuntimeError("runtime")
        return "ok"
    except TypeError as e:
        return str(e)
    except ValueError:
        return "value"
    except:
        return "other"


def test_except_chain_and_bare_except() -> None:
    assert classify_exception(0) == "value"
    assert classify_exception(1) == "type"
    assert classify_exception(2) == "other"
    assert classify_exception(3) == "ok"


def test_dynamic_float_tuple_index() -> None:
    vals: tuple[float, float, float] = (1.25, 2.5, 3.75)
    idx: int = 1
    got: float = vals[idx]
    assert got == 2.5


def test_for_iter_break_continue_else() -> None:
    total: int = 0
    for n in TinyIter(4):
        if n == 2:
            continue
        total = total + n
    else:
        total = total + 100
    assert total == 108

    seen: int = 0
    for n in TinyIter(10):
        if n == 4:
            break
        seen = seen + 1
    assert seen == 3


def test_personality_paths_in_while_else() -> None:
    v: int = 0
    while v < 0:
        v = v + 1
    else:
        for n in TinyIter(1):
            v = v + n
    assert v == 1

    msg_len: int = 0
    while msg_len < 0:
        msg_len = msg_len + 1
    else:
        try:
            raise Exception("boom")
        except Exception as e:
            msg_len = len(str(e))
    assert msg_len == 4


def test_dead_bare_raise_is_valid() -> None:
    marker: int = 9
    if False:
        raise
    assert marker == 9


def run_tests() -> None:
    test_indirect_void_call()
    test_for_range_else()
    test_try_finally_without_except()
    test_except_chain_and_bare_except()
    test_dynamic_float_tuple_index()
    test_for_iter_break_continue_else()
    test_personality_paths_in_while_else()
    test_dead_bare_raise_is_valid()