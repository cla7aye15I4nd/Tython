def test_str_literal() -> None:
    s: str = "hello"


def test_str_empty() -> None:
    s: str = ""
    print('CHECK test_str lhs:', len(s))
    print('CHECK test_str rhs:', 0)
    assert len(s) == 0


def test_str_concat() -> None:
    a: str = "hello"
    b: str = " world"
    c: str = a + b
    print('CHECK test_str lhs:', c)
    print('CHECK test_str rhs:', 'hello world')
    assert c == "hello world"


def test_str_repeat() -> None:
    s: str = "ab"
    r: str = s * 3
    print('CHECK test_str lhs:', r)
    print('CHECK test_str rhs:', 'ababab')
    assert r == "ababab"


def test_str_repeat_reverse() -> None:
    s: str = "xy"
    r: str = 2 * s
    print('CHECK test_str lhs:', r)
    print('CHECK test_str rhs:', 'xyxy')
    assert r == "xyxy"


def test_str_repeat_zero() -> None:
    s: str = "abc"
    r: str = s * 0
    print('CHECK test_str lhs:', len(r))
    print('CHECK test_str rhs:', 0)
    assert len(r) == 0


def test_str_comparison() -> None:
    print('CHECK test_str lhs:', 'abc')
    print('CHECK test_str rhs:', 'abc')
    assert "abc" == "abc"
    print('CHECK test_str assert expr:', '"abc" != "def"')
    assert "abc" != "def"
    print('CHECK test_str assert expr:', '"abc" < "abd"')
    assert "abc" < "abd"
    print('CHECK test_str assert expr:', '"b" > "a"')
    assert "b" > "a"
    print('CHECK test_str assert expr:', '"abc" <= "abc"')
    assert "abc" <= "abc"
    print('CHECK test_str assert expr:', '"abc" >= "abc"')
    assert "abc" >= "abc"
    print('CHECK test_str assert expr:', '"a" <= "b"')
    assert "a" <= "b"
    print('CHECK test_str assert expr:', '"b" >= "a"')
    assert "b" >= "a"


def test_str_len() -> None:
    s: str = "hello"
    print('CHECK test_str lhs:', len(s))
    print('CHECK test_str rhs:', 5)
    assert len(s) == 5
    print('CHECK test_str lhs:', len(""))
    print('CHECK test_str rhs:', 0)
    assert len("") == 0
    print('CHECK test_str lhs:', len("a"))
    print('CHECK test_str rhs:', 1)
    assert len("a") == 1


def test_str_from_int() -> None:
    s: str = str(42)
    print('CHECK test_str lhs:', s)
    print('CHECK test_str rhs:', '42')
    assert s == "42"
    print('CHECK test_str lhs:', str(0))
    print('CHECK test_str rhs:', '0')
    assert str(0) == "0"
    print('CHECK test_str lhs:', str(0 - 1))
    print('CHECK test_str rhs:', '-1')
    assert str(0 - 1) == "-1"


def test_str_from_bool() -> None:
    print('CHECK test_str lhs:', str(True))
    print('CHECK test_str rhs:', 'True')
    assert str(True) == "True"
    print('CHECK test_str lhs:', str(False))
    print('CHECK test_str rhs:', 'False')
    assert str(False) == "False"


def test_str_from_float() -> None:
    s: str = str(3.14)


def test_str_identity() -> None:
    s: str = "hello"
    t: str = str(s)
    print('CHECK test_str lhs:', t)
    print('CHECK test_str rhs:', 'hello')
    assert t == "hello"


def test_str_truthiness() -> None:
    if "hello":
        print("truthy")
    if "":
        print("should not print")


def test_str_assert() -> None:
    print('CHECK test_str assert expr:', '"hello"')
    assert "hello"


def test_str_methods_strip_split_join() -> None:
    raw: str = "  a,b,c  "
    stripped: str = raw.strip()
    parts: list[str] = stripped.split(",")
    joined: str = "-".join(parts)
    print('CHECK test_str lhs:', stripped)
    print('CHECK test_str rhs:', 'a,b,c')
    assert stripped == "a,b,c"
    print('CHECK test_str lhs:', parts)
    print('CHECK test_str rhs:', ['a', 'b', 'c'])
    assert parts == ["a", "b", "c"]
    print('CHECK test_str lhs:', joined)
    print('CHECK test_str rhs:', 'a-b-c')
    assert joined == "a-b-c"


def run_tests() -> None:
    test_str_literal()
    test_str_empty()
    test_str_concat()
    test_str_repeat()
    test_str_repeat_reverse()
    test_str_repeat_zero()
    test_str_comparison()
    test_str_len()
    test_str_from_int()
    test_str_from_bool()
    test_str_from_float()
    test_str_identity()
    test_str_truthiness()
    test_str_assert()
    test_str_methods_strip_split_join()
