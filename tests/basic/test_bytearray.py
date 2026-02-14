def test_bytearray_empty() -> None:
    ba: bytearray = bytearray()
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 0)
    assert len(ba) == 0


def test_bytearray_from_int() -> None:
    ba: bytearray = bytearray(5)
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 5)
    assert len(ba) == 5


def test_bytearray_from_bytes() -> None:
    ba: bytearray = bytearray(b"hello")
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 5)
    assert len(ba) == 5


def test_bytearray_concat() -> None:
    a: bytearray = bytearray(b"hello")
    b: bytearray = bytearray(b" world")
    c: bytearray = a + b
    print('CHECK test_bytearray lhs:', c)
    print('CHECK test_bytearray rhs:', bytearray(b'hello world'))
    assert c == bytearray(b"hello world")


def test_bytearray_repeat() -> None:
    ba: bytearray = bytearray(b"ab")
    r: bytearray = ba * 3
    print('CHECK test_bytearray lhs:', r)
    print('CHECK test_bytearray rhs:', bytearray(b'ababab'))
    assert r == bytearray(b"ababab")


def test_bytearray_comparison() -> None:
    print('CHECK test_bytearray lhs:', bytearray(b'abc'))
    print('CHECK test_bytearray rhs:', bytearray(b'abc'))
    assert bytearray(b"abc") == bytearray(b"abc")
    print('CHECK test_bytearray assert expr:', 'bytearray(b"abc") != bytearray(b"def")')
    assert bytearray(b"abc") != bytearray(b"def")
    print('CHECK test_bytearray assert expr:', 'bytearray(b"abc") < bytearray(b"abd")')
    assert bytearray(b"abc") < bytearray(b"abd")


def test_bytearray_len() -> None:
    ba: bytearray = bytearray(b"hello")
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 5)
    assert len(ba) == 5


def test_bytearray_append() -> None:
    ba: bytearray = bytearray(b"hi")
    ba.append(33)
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 3)
    assert len(ba) == 3


def test_bytearray_extend() -> None:
    ba: bytearray = bytearray(b"hi")
    ba.extend(b" there")
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 8)
    assert len(ba) == 8


def test_bytearray_clear() -> None:
    ba: bytearray = bytearray(b"hello")
    ba.clear()
    print('CHECK test_bytearray lhs:', len(ba))
    print('CHECK test_bytearray rhs:', 0)
    assert len(ba) == 0


def test_bytearray_identity() -> None:
    ba: bytearray = bytearray(b"hello")
    ba2: bytearray = bytearray(ba)
    print('CHECK test_bytearray lhs:', ba2)
    print('CHECK test_bytearray rhs:', bytearray(b'hello'))
    assert ba2 == bytearray(b"hello")


def test_bytearray_truthiness() -> None:
    if bytearray(b"hello"):
        print("truthy")
    if bytearray():
        print("should not print")


def test_bytearray_assert() -> None:
    print('CHECK test_bytearray assert expr:', 'bytearray(b"hello")')
    assert bytearray(b"hello")


def test_bytearray_methods_basic() -> None:
    raw: bytearray = bytearray(b" a,b ")
    stripped: bytearray = raw.strip(b" ")
    parts: list[bytearray] = stripped.split(b",")
    joined: bytearray = bytearray(b"-").join(parts)
    decoded: str = raw.decode()

    print('CHECK test_bytearray lhs:', stripped)
    print('CHECK test_bytearray rhs:', bytearray(b'a,b'))
    assert stripped == bytearray(b"a,b")
    print('CHECK test_bytearray lhs:', len(parts))
    print('CHECK test_bytearray rhs:', 2)
    assert len(parts) == 2
    print('CHECK test_bytearray lhs:', parts[0])
    print('CHECK test_bytearray rhs:', bytearray(b'a'))
    assert parts[0] == bytearray(b"a")
    print('CHECK test_bytearray lhs:', parts[1])
    print('CHECK test_bytearray rhs:', bytearray(b'b'))
    assert parts[1] == bytearray(b"b")
    print('CHECK test_bytearray lhs:', joined)
    print('CHECK test_bytearray rhs:', bytearray(b'a-b'))
    assert joined == bytearray(b"a-b")
    print('CHECK test_bytearray lhs:', decoded)
    print('CHECK test_bytearray rhs:', ' a,b ')
    assert decoded == " a,b "


def test_bytearray_methods_matrix() -> None:
    src: bytearray = bytearray(b"ab-ab")
    centered: bytearray = bytearray(b"ab").center(4, b"_")
    cnt: int = src.count(b"ab")
    found: int = src.find(b"-")
    idx: int = src.index(b"-")
    rf: int = src.rfind(b"ab")
    ridx: int = src.rindex(b"ab")
    hx: str = bytearray(b"\x0f\x10").hex()
    from_hx: bytearray = bytearray(b"").fromhex("0f10")
    rep: bytearray = src.replace(b"-", b":")
    part: tuple[bytearray, bytearray, bytearray] = src.partition(b"-")
    rpart: tuple[bytearray, bytearray, bytearray] = src.rpartition(b"-")
    splitv: list[bytearray] = src.split(b"-")
    rsplitv: list[bytearray] = src.rsplit(b"-")
    starts: bool = src.startswith(b"ab")
    ends: bool = src.endswith(b"ab")
    lower_ok: bool = bytearray(b"ab").islower()
    upper_ok: bool = bytearray(b"AB").isupper()
    title_ok: bool = bytearray(b"Ab Cd").istitle()
    digits_ok: bool = bytearray(b"123").isdigit()
    alpha_ok: bool = bytearray(b"abc").isalpha()
    alnum_ok: bool = bytearray(b"abc123").isalnum()
    ascii_ok: bool = bytearray(b"abc").isascii()
    space_ok: bool = bytearray(b" \t").isspace()
    tabbed: bytearray = bytearray(b"a\tb").expandtabs(4)
    stripped_l: bytearray = bytearray(b"__ab").lstrip(b"_")
    stripped_r: bytearray = bytearray(b"ab__").rstrip(b"_")
    stripped: bytearray = bytearray(b"__ab__").strip(b"_")
    ljusted: bytearray = bytearray(b"ab").ljust(4, b"_")
    rjusted: bytearray = bytearray(b"ab").rjust(4, b"_")
    titled: bytearray = bytearray(b"ab cd").title()
    swapd: bytearray = bytearray(b"Ab").swapcase()
    upperd: bytearray = bytearray(b"ab").upper()
    lowerd: bytearray = bytearray(b"AB").lower()
    zp: bytearray = bytearray(b"-7").zfill(4)
    pref: bytearray = bytearray(b"foobar").removeprefix(b"foo")
    suff: bytearray = bytearray(b"foobar").removesuffix(b"bar")
    trans_tbl: bytes = bytearray(b"").maketrans(b"a", b"z")
    transed: bytearray = bytearray(b"aba").translate(trans_tbl)
    lines: list[bytearray] = bytearray(b"a\nb").splitlines()

    print('CHECK test_bytearray lhs:', centered)
    print('CHECK test_bytearray rhs:', bytearray(b'_ab_'))
    assert centered == bytearray(b"_ab_")
    print('CHECK test_bytearray lhs:', cnt)
    print('CHECK test_bytearray rhs:', 2)
    assert cnt == 2
    print('CHECK test_bytearray lhs:', found)
    print('CHECK test_bytearray rhs:', 2)
    assert found == 2
    print('CHECK test_bytearray lhs:', idx)
    print('CHECK test_bytearray rhs:', 2)
    assert idx == 2
    print('CHECK test_bytearray lhs:', rf)
    print('CHECK test_bytearray rhs:', 3)
    assert rf == 3
    print('CHECK test_bytearray lhs:', ridx)
    print('CHECK test_bytearray rhs:', 3)
    assert ridx == 3
    assert hx == "0f10"
    assert from_hx == bytearray(b"\x0f\x10")
    assert rep == bytearray(b"ab:ab")
    assert part == (bytearray(b"ab"), bytearray(b"-"), bytearray(b"ab"))
    assert rpart == (bytearray(b"ab"), bytearray(b"-"), bytearray(b"ab"))
    assert splitv == [bytearray(b"ab"), bytearray(b"ab")]
    assert rsplitv == [bytearray(b"ab"), bytearray(b"ab")]
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
    assert tabbed == bytearray(b"a   b")
    assert stripped_l == bytearray(b"ab")
    assert stripped_r == bytearray(b"ab")
    assert stripped == bytearray(b"ab")
    assert ljusted == bytearray(b"ab__")
    assert rjusted == bytearray(b"__ab")
    assert titled == bytearray(b"Ab Cd")
    assert swapd == bytearray(b"aB")
    assert upperd == bytearray(b"AB")
    assert lowerd == bytearray(b"ab")
    assert zp == bytearray(b"-007")
    assert pref == bytearray(b"bar")
    assert suff == bytearray(b"foo")
    assert transed == bytearray(b"zbz")
    assert lines == [bytearray(b"a"), bytearray(b"b")]


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
    test_bytearray_methods_basic()
    test_bytearray_methods_matrix()
