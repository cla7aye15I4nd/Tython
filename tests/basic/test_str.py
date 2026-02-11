def test_str_literal() -> None:
    s: str = "hello"
    print(s)


def test_str_empty() -> None:
    s: str = ""
    print(len(s))
    assert len(s) == 0


def test_str_concat() -> None:
    a: str = "hello"
    b: str = " world"
    c: str = a + b
    print(c)
    assert c == "hello world"


def test_str_repeat() -> None:
    s: str = "ab"
    r: str = s * 3
    print(r)
    assert r == "ababab"


def test_str_repeat_reverse() -> None:
    s: str = "xy"
    r: str = 2 * s
    print(r)
    assert r == "xyxy"


def test_str_repeat_zero() -> None:
    s: str = "abc"
    r: str = s * 0
    assert len(r) == 0


def test_str_comparison() -> None:
    assert "abc" == "abc"
    assert "abc" != "def"
    assert "abc" < "abd"
    assert "b" > "a"
    assert "abc" <= "abc"
    assert "abc" >= "abc"
    assert "a" <= "b"
    assert "b" >= "a"


def test_str_len() -> None:
    s: str = "hello"
    print(len(s))
    assert len(s) == 5
    assert len("") == 0
    assert len("a") == 1


def test_str_from_int() -> None:
    s: str = str(42)
    print(s)
    assert s == "42"
    assert str(0) == "0"
    assert str(0 - 1) == "-1"


def test_str_from_bool() -> None:
    assert str(True) == "True"
    assert str(False) == "False"


def test_str_truthiness() -> None:
    if "hello":
        print("truthy")
    if "":
        print("should not print")


def test_str_assert() -> None:
    assert "hello"


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
    test_str_truthiness()
    test_str_assert()
