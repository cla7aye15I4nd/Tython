def test_bytes_literal() -> None:
    b: bytes = b"hello"
    print(b)


def test_bytes_empty() -> None:
    b: bytes = b""
    assert len(b) == 0


def test_bytes_concat() -> None:
    a: bytes = b"hello"
    b: bytes = b" world"
    c: bytes = a + b
    print(c)
    assert c == b"hello world"


def test_bytes_repeat() -> None:
    b: bytes = b"ab"
    r: bytes = b * 3
    print(r)
    assert r == b"ababab"


def test_bytes_repeat_reverse() -> None:
    b: bytes = b"xy"
    r: bytes = 2 * b
    assert r == b"xyxy"


def test_bytes_comparison() -> None:
    assert b"abc" == b"abc"
    assert b"abc" != b"def"
    assert b"abc" < b"abd"
    assert b"b" > b"a"


def test_bytes_len() -> None:
    b: bytes = b"hello"
    print(len(b))
    assert len(b) == 5
    assert len(b"") == 0


def test_bytes_from_int() -> None:
    b: bytes = bytes(5)
    assert len(b) == 5


def test_bytes_identity() -> None:
    b: bytes = b"hello"
    c: bytes = bytes(b)
    assert c == b"hello"


def test_bytes_truthiness() -> None:
    if b"hello":
        print("truthy")
    if b"":
        print("should not print")


def test_bytes_assert() -> None:
    assert b"hello"


def run_tests() -> None:
    test_bytes_literal()
    test_bytes_empty()
    test_bytes_concat()
    test_bytes_repeat()
    test_bytes_repeat_reverse()
    test_bytes_comparison()
    test_bytes_len()
    test_bytes_from_int()
    test_bytes_identity()
    test_bytes_truthiness()
    test_bytes_assert()
