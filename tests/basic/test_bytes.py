def test_bytes_literal() -> None:
    b: bytes = b"hello"
    print(b)


def test_bytes_empty() -> None:
    b: bytes = b""
    print('CHECK test_bytes lhs expr:', 'len(b)')
    print('CHECK test_bytes rhs:', 0)
    assert len(b) == 0


def test_bytes_concat() -> None:
    a: bytes = b"hello"
    b: bytes = b" world"
    c: bytes = a + b
    print(c)
    print('CHECK test_bytes lhs:', c)
    print('CHECK test_bytes rhs:', b'hello world')
    assert c == b"hello world"


def test_bytes_repeat() -> None:
    b: bytes = b"ab"
    r: bytes = b * 3
    print(r)
    print('CHECK test_bytes lhs:', r)
    print('CHECK test_bytes rhs:', b'ababab')
    assert r == b"ababab"


def test_bytes_repeat_reverse() -> None:
    b: bytes = b"xy"
    r: bytes = 2 * b
    print('CHECK test_bytes lhs:', r)
    print('CHECK test_bytes rhs:', b'xyxy')
    assert r == b"xyxy"


def test_bytes_comparison() -> None:
    print('CHECK test_bytes lhs:', b'abc')
    print('CHECK test_bytes rhs:', b'abc')
    assert b"abc" == b"abc"
    print('CHECK test_bytes assert expr:', 'b"abc" != b"def"')
    assert b"abc" != b"def"
    print('CHECK test_bytes assert expr:', 'b"abc" < b"abd"')
    assert b"abc" < b"abd"
    print('CHECK test_bytes assert expr:', 'b"b" > b"a"')
    assert b"b" > b"a"


def test_bytes_len() -> None:
    b: bytes = b"hello"
    print(len(b))
    print('CHECK test_bytes lhs expr:', 'len(b)')
    print('CHECK test_bytes rhs:', 5)
    assert len(b) == 5
    print('CHECK test_bytes lhs expr:', "len(b'')")
    print('CHECK test_bytes rhs:', 0)
    assert len(b"") == 0


def test_bytes_from_int() -> None:
    b: bytes = bytes(5)
    print('CHECK test_bytes lhs expr:', 'len(b)')
    print('CHECK test_bytes rhs:', 5)
    assert len(b) == 5


def test_bytes_identity() -> None:
    b: bytes = b"hello"
    c: bytes = bytes(b)
    print('CHECK test_bytes lhs:', c)
    print('CHECK test_bytes rhs:', b'hello')
    assert c == b"hello"


def test_bytes_truthiness() -> None:
    if b"hello":
        print("truthy")
    if b"":
        print("should not print")


def test_bytes_assert() -> None:
    print('CHECK test_bytes assert expr:', 'b"hello"')
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
