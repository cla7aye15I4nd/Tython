def test_bytearray_empty() -> None:
    ba: bytearray = bytearray()
    assert len(ba) == 0


def test_bytearray_from_int() -> None:
    ba: bytearray = bytearray(5)
    assert len(ba) == 5


def test_bytearray_from_bytes() -> None:
    ba: bytearray = bytearray(b"hello")
    print(ba)
    assert len(ba) == 5


def test_bytearray_concat() -> None:
    a: bytearray = bytearray(b"hello")
    b: bytearray = bytearray(b" world")
    c: bytearray = a + b
    assert c == bytearray(b"hello world")


def test_bytearray_repeat() -> None:
    ba: bytearray = bytearray(b"ab")
    r: bytearray = ba * 3
    assert r == bytearray(b"ababab")


def test_bytearray_comparison() -> None:
    assert bytearray(b"abc") == bytearray(b"abc")
    assert bytearray(b"abc") != bytearray(b"def")
    assert bytearray(b"abc") < bytearray(b"abd")


def test_bytearray_len() -> None:
    ba: bytearray = bytearray(b"hello")
    assert len(ba) == 5


def test_bytearray_append() -> None:
    ba: bytearray = bytearray(b"hi")
    ba.append(33)
    assert len(ba) == 3


def test_bytearray_extend() -> None:
    ba: bytearray = bytearray(b"hi")
    ba.extend(b" there")
    assert len(ba) == 8


def test_bytearray_clear() -> None:
    ba: bytearray = bytearray(b"hello")
    ba.clear()
    assert len(ba) == 0


def test_bytearray_identity() -> None:
    ba: bytearray = bytearray(b"hello")
    ba2: bytearray = bytearray(ba)
    assert ba2 == bytearray(b"hello")


def test_bytearray_truthiness() -> None:
    if bytearray(b"hello"):
        print("truthy")
    if bytearray():
        print("should not print")


def test_bytearray_assert() -> None:
    assert bytearray(b"hello")


def run_tests() -> None:
    test_bytearray_empty()
    test_bytearray_from_int()
    test_bytearray_from_bytes()
    test_bytearray_concat()
    test_bytearray_repeat()
    test_bytearray_comparison()
    test_bytearray_len()
    test_bytearray_append()
    test_bytearray_extend()
    test_bytearray_clear()
    test_bytearray_identity()
    test_bytearray_truthiness()
    test_bytearray_assert()
