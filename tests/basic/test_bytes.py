def test_bytes_literal() -> None:
    b: bytes = b"hello"


def test_bytes_empty() -> None:
    b: bytes = b""
    print('CHECK test_bytes lhs:', len(b))
    print('CHECK test_bytes rhs:', 0)
    assert len(b) == 0


def test_bytes_concat() -> None:
    a: bytes = b"hello"
    b: bytes = b" world"
    c: bytes = a + b
    print('CHECK test_bytes lhs:', c)
    print('CHECK test_bytes rhs:', b'hello world')
    assert c == b"hello world"


def test_bytes_repeat() -> None:
    b: bytes = b"ab"
    r: bytes = b * 3
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
    print('CHECK test_bytes lhs:', b"abc")
    print('CHECK test_bytes rhs:', b"def")
    assert b"abc" != b"def"
    print('CHECK test_bytes lhs:', b"abc")
    print('CHECK test_bytes rhs:', b"abd")
    assert b"abc" < b"abd"
    print('CHECK test_bytes lhs:', b"b")
    print('CHECK test_bytes rhs:', b"a")
    assert b"b" > b"a"


def test_bytes_len() -> None:
    b: bytes = b"hello"
    print('CHECK test_bytes lhs:', len(b))
    print('CHECK test_bytes rhs:', 5)
    assert len(b) == 5
    print('CHECK test_bytes lhs:', len(b""))
    print('CHECK test_bytes rhs:', 0)
    assert len(b"") == 0


def test_bytes_from_int() -> None:
    b: bytes = bytes(5)
    print('CHECK test_bytes lhs:', len(b))
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


def test_bytes_methods_basic() -> None:
    raw: bytes = b" a,b "
    stripped: bytes = raw.strip(b" ")
    parts: list[bytes] = stripped.split(b",")
    joined: bytes = b"-".join(parts)
    decoded: str = raw.decode()

    print('CHECK test_bytes lhs:', stripped)
    print('CHECK test_bytes rhs:', b'a,b')
    assert stripped == b"a,b"
    print('CHECK test_bytes lhs:', parts)
    print('CHECK test_bytes rhs:', [b'a', b'b'])
    assert parts == [b"a", b"b"]
    print('CHECK test_bytes lhs:', joined)
    print('CHECK test_bytes rhs:', b'a-b')
    assert joined == b"a-b"
    print('CHECK test_bytes lhs:', decoded)
    print('CHECK test_bytes rhs:', ' a,b ')
    assert decoded == " a,b "


def test_bytes_methods_matrix() -> None:
    src: bytes = b"ab-ab"
    centered: bytes = b"ab".center(4, b"_")
    cnt: int = src.count(b"ab")
    found: int = src.find(b"-")
    idx: int = src.index(b"-")
    rf: int = src.rfind(b"ab")
    ridx: int = src.rindex(b"ab")
    hx: str = b"\x0f\x10".hex()
    from_hx: bytes = b"".fromhex("0f10")
    rep: bytes = src.replace(b"-", b":")
    part: tuple[bytes, bytes, bytes] = src.partition(b"-")
    rpart: tuple[bytes, bytes, bytes] = src.rpartition(b"-")
    splitv: list[bytes] = src.split(b"-")
    rsplitv: list[bytes] = src.rsplit(b"-")
    starts: bool = src.startswith(b"ab")
    ends: bool = src.endswith(b"ab")
    lower_ok: bool = b"ab".islower()
    upper_ok: bool = b"AB".isupper()
    title_ok: bool = b"Ab Cd".istitle()
    digits_ok: bool = b"123".isdigit()
    alpha_ok: bool = b"abc".isalpha()
    alnum_ok: bool = b"abc123".isalnum()
    ascii_ok: bool = b"abc".isascii()
    space_ok: bool = b" \t".isspace()
    tabbed: bytes = b"a\tb".expandtabs(4)
    stripped_l: bytes = b"__ab".lstrip(b"_")
    stripped_r: bytes = b"ab__".rstrip(b"_")
    stripped: bytes = b"__ab__".strip(b"_")
    ljusted: bytes = b"ab".ljust(4, b"_")
    rjusted: bytes = b"ab".rjust(4, b"_")
    titled: bytes = b"ab cd".title()
    swapd: bytes = b"Ab".swapcase()
    upperd: bytes = b"ab".upper()
    lowerd: bytes = b"AB".lower()
    zp: bytes = b"-7".zfill(4)
    pref: bytes = b"foobar".removeprefix(b"foo")
    suff: bytes = b"foobar".removesuffix(b"bar")
    trans_tbl: bytes = b"".maketrans(b"a", b"z")
    transed: bytes = b"aba".translate(trans_tbl)
    lines: list[bytes] = b"a\nb".splitlines()

    print('CHECK test_bytes lhs:', centered)
    print('CHECK test_bytes rhs:', b'_ab_')
    assert centered == b"_ab_"
    print('CHECK test_bytes lhs:', cnt)
    print('CHECK test_bytes rhs:', 2)
    assert cnt == 2
    print('CHECK test_bytes lhs:', found)
    print('CHECK test_bytes rhs:', 2)
    assert found == 2
    print('CHECK test_bytes lhs:', idx)
    print('CHECK test_bytes rhs:', 2)
    assert idx == 2
    print('CHECK test_bytes lhs:', rf)
    print('CHECK test_bytes rhs:', 3)
    assert rf == 3
    print('CHECK test_bytes lhs:', ridx)
    print('CHECK test_bytes rhs:', 3)
    assert ridx == 3
    assert hx == "0f10"
    assert from_hx == b"\x0f\x10"
    assert rep == b"ab:ab"
    assert part == (b"ab", b"-", b"ab")
    assert rpart == (b"ab", b"-", b"ab")
    assert splitv == [b"ab", b"ab"]
    assert rsplitv == [b"ab", b"ab"]
    assert starts == True
    assert ends == True
    assert lower_ok == True
    assert upper_ok == True
    assert title_ok == True
    assert digits_ok == True
    assert alpha_ok == True
    assert alnum_ok == True
    assert ascii_ok == True
    assert space_ok == True
    assert tabbed == b"a   b"
    assert stripped_l == b"ab"
    assert stripped_r == b"ab"
    assert stripped == b"ab"
    assert ljusted == b"ab__"
    assert rjusted == b"__ab"
    assert titled == b"Ab Cd"
    assert swapd == b"aB"
    assert upperd == b"AB"
    assert lowerd == b"ab"
    assert zp == b"-007"
    assert pref == b"bar"
    assert suff == b"foo"
    assert transed == b"zbz"
    assert lines == [b"a", b"b"]


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
    test_bytes_methods_basic()
    test_bytes_methods_matrix()
