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
    print('CHECK test_codegen_edges lhs:', total)
    print('CHECK test_codegen_edges rhs:', 16)
    assert total == 16


def test_try_finally_without_except() -> None:
    events: list[int] = []
    try:
        events.append(1)
    finally:
        events.append(2)
    print('CHECK test_codegen_edges lhs:', events)
    print('CHECK test_codegen_edges rhs:', [1, 2])
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
    print('CHECK test_codegen_edges lhs expr:', 'classify_exception(0)')
    print('CHECK test_codegen_edges rhs:', 'value')
    assert classify_exception(0) == "value"
    print('CHECK test_codegen_edges lhs expr:', 'classify_exception(1)')
    print('CHECK test_codegen_edges rhs:', 'type')
    assert classify_exception(1) == "type"
    print('CHECK test_codegen_edges lhs expr:', 'classify_exception(2)')
    print('CHECK test_codegen_edges rhs:', 'other')
    assert classify_exception(2) == "other"
    print('CHECK test_codegen_edges lhs expr:', 'classify_exception(3)')
    print('CHECK test_codegen_edges rhs:', 'ok')
    assert classify_exception(3) == "ok"


def test_dynamic_float_tuple_index() -> None:
    vals: tuple[float, float, float] = (1.25, 2.5, 3.75)
    idx: int = 1
    got: float = vals[idx]
    print('CHECK test_codegen_edges lhs:', got)
    print('CHECK test_codegen_edges rhs:', 2.5)
    assert got == 2.5


def test_for_iter_break_continue_else() -> None:
    total: int = 0
    for n in TinyIter(4):
        if n == 2:
            continue
        total = total + n
    else:
        total = total + 100
    print('CHECK test_codegen_edges lhs:', total)
    print('CHECK test_codegen_edges rhs:', 108)
    assert total == 108

    seen: int = 0
    for n in TinyIter(10):
        if n == 4:
            break
        seen = seen + 1
    print('CHECK test_codegen_edges lhs:', seen)
    print('CHECK test_codegen_edges rhs:', 3)
    assert seen == 3


def test_personality_paths_in_while_else() -> None:
    v: int = 0
    while v < 0:
        v = v + 1
    else:
        for n in TinyIter(1):
            v = v + n
    print('CHECK test_codegen_edges lhs:', v)
    print('CHECK test_codegen_edges rhs:', 1)
    assert v == 1

    msg_len: int = 0
    while msg_len < 0:
        msg_len = msg_len + 1
    else:
        try:
            raise Exception("boom")
        except Exception as e:
            msg_len = len(str(e))
    print('CHECK test_codegen_edges lhs:', msg_len)
    print('CHECK test_codegen_edges rhs:', 4)
    assert msg_len == 4


def test_dead_bare_raise_is_valid() -> None:
    marker: int = 9
    if False:
        raise
    print('CHECK test_codegen_edges lhs:', marker)
    print('CHECK test_codegen_edges rhs:', 9)
    assert marker == 9


def test_trycatch_personality_in_while_else() -> None:
    x: int = 0
    while x < 0:
        x = x + 1
    else:
        try:
            raise Exception("catch me")
        except Exception:
            x = 99
    print('CHECK test_codegen_edges lhs:', x)
    print('CHECK test_codegen_edges rhs:', 99)
    assert x == 99


def add_floats(a: float, b: float) -> float:
    return a + b


def test_float_args_in_try_invoke() -> None:
    result: float = 0.0
    try:
        result = add_floats(1.5, 2.5)
    except Exception:
        result = -1.0
    print('CHECK test_codegen_edges lhs:', result)
    print('CHECK test_codegen_edges rhs:', 4.0)
    assert result == 4.0


def test_try_inside_if_personality() -> None:
    x: int = 0
    if True:
        try:
            x = 42
        except Exception:
            x = -1
    print('CHECK test_codegen_edges lhs:', x)
    print('CHECK test_codegen_edges rhs:', 42)
    assert x == 42


def test_for_iter_inside_if_personality() -> None:
    total: int = 0
    if True:
        for n in TinyIter(3):
            total = total + n
    print('CHECK test_codegen_edges lhs:', total)
    print('CHECK test_codegen_edges rhs:', 6)
    assert total == 6


def run_tests() -> None:
    test_indirect_void_call()
    test_for_range_else()
    test_try_finally_without_except()
    test_except_chain_and_bare_except()
    test_dynamic_float_tuple_index()
    test_for_iter_break_continue_else()
    test_personality_paths_in_while_else()
    test_dead_bare_raise_is_valid()
    test_trycatch_personality_in_while_else()
    test_float_args_in_try_invoke()
    test_try_inside_if_personality()
    test_for_iter_inside_if_personality()