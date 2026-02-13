def test_bytes_core_ops() -> None:
    b: bytes = b"hello"
    print("bytes core len:", len(b))
    assert len(b) == 5

    c: bytes = b + b"!"
    print("bytes core concat:", c)
    assert c == b"hello!"

    r1: bytes = b"ab" * 3
    r2: bytes = 2 * b"xy"
    print("bytes core repeat1:", r1)
    print("bytes core repeat2:", r2)
    assert r1 == b"ababab"
    assert r2 == b"xyxy"


def test_bytes_capitalize_center_count_decode() -> None:
    cap: bytes = b"hELLo".capitalize()
    cen: bytes = b"x".center(5, b"-")
    cnt: int = b"aaaa".count(b"aa")
    dec: str = b"abc".decode()

    print("bytes capitalize:", cap)
    print("bytes center:", cen)
    print("bytes count:", cnt)
    print("bytes decode:", dec)

    assert cap == b"Hello"
    assert cen == b"--x--"
    assert cnt == 2
    assert dec == "abc"


def test_bytes_endswith_expandtabs_find_fromhex_hex_index() -> None:
    ew: bool = b"alpha.py".endswith(b".py")
    ex: bytes = b"a\tb".expandtabs(4)
    fd: int = b"banana".find(b"na")
    fh: bytes = b"ignored".fromhex("61 62 63")
    hx: str = b"abc".hex()
    ix: int = b"banana".index(b"na")

    print("bytes endswith:", ew)
    print("bytes expandtabs:", ex)
    print("bytes find:", fd)
    print("bytes fromhex:", fh)
    print("bytes hex:", hx)
    print("bytes index:", ix)

    assert ew
    assert ex == b"a   b"
    assert fd == 2
    assert fh == b"abc"
    assert hx == "616263"
    assert ix == 2


def test_bytes_predicates() -> None:
    p1: bool = b"abc123".isalnum()
    p2: bool = b"abc".isalpha()
    p3: bool = b"abc".isascii()
    p4: bool = b"\xff".isascii()
    p5: bool = b"123".isdigit()
    p6: bool = b"abc".islower()
    p7: bool = b" \t\n".isspace()
    p8: bool = b"Hello World".istitle()
    p9: bool = b"ABC".isupper()

    print("bytes isalnum:", p1)
    print("bytes isalpha:", p2)
    print("bytes isascii ascii:", p3)
    print("bytes isascii non-ascii:", p4)
    print("bytes isdigit:", p5)
    print("bytes islower:", p6)
    print("bytes isspace:", p7)
    print("bytes istitle:", p8)
    print("bytes isupper:", p9)

    assert p1
    assert p2
    assert p3
    assert not p4
    assert p5
    assert p6
    assert p7
    assert p8
    assert p9


def test_bytes_join_ljust_lower_lstrip() -> None:
    parts: list[bytes] = [b"a", b"b", b"c"]
    jn: bytes = b"-".join(parts)
    lj: bytes = b"x".ljust(4, b".")
    lo: bytes = b"AbC".lower()
    ls: bytes = b"000123".lstrip(b"0")

    print("bytes join:", jn)
    print("bytes ljust:", lj)
    print("bytes lower:", lo)
    print("bytes lstrip:", ls)

    assert jn == b"a-b-c"
    assert lj == b"x..."
    assert lo == b"abc"
    assert ls == b"123"


def test_bytes_maketrans_partition_prefix_suffix_replace() -> None:
    table: bytes = b"_".maketrans(b"abc", b"xyz")
    print("bytes maketrans len:", len(table))
    assert len(table) == 256

    p: tuple[bytes, bytes, bytes] = b"key=value".partition(b"=")
    print("bytes partition:", p[0], p[1], p[2])
    assert p[0] == b"key"
    assert p[1] == b"="
    assert p[2] == b"value"

    rp: bytes = b"prefix_data".removeprefix(b"prefix_")
    rs: bytes = b"data.txt".removesuffix(b".txt")
    rr: bytes = b"one one".replace(b"one", b"two")

    print("bytes removeprefix:", rp)
    print("bytes removesuffix:", rs)
    print("bytes replace:", rr)

    assert rp == b"data"
    assert rs == b"data"
    assert rr == b"two two"


def test_bytes_rfind_rindex_rjust_rpartition() -> None:
    rf: int = b"banana".rfind(b"na")
    ri: int = b"banana".rindex(b"na")
    rj: bytes = b"x".rjust(4, b".")
    p: tuple[bytes, bytes, bytes] = b"a=b=c".rpartition(b"=")

    print("bytes rfind:", rf)
    print("bytes rindex:", ri)
    print("bytes rjust:", rj)
    print("bytes rpartition:", p[0], p[1], p[2])

    assert rf == 4
    assert ri == 4
    assert rj == b"...x"
    assert p[0] == b"a=b"
    assert p[1] == b"="
    assert p[2] == b"c"


def test_bytes_rsplit_rstrip_split_splitlines() -> None:
    rs: list[bytes] = b"a,b,c".rsplit(b",")
    rt: bytes = b"abc...".rstrip(b".")
    sp: list[bytes] = b"a,b,c".split(b",")
    sl: list[bytes] = b"a\nb\r\nc".splitlines()

    print("bytes rsplit:", rs)
    print("bytes rstrip:", rt)
    print("bytes split:", sp)
    print("bytes splitlines:", sl)

    assert rs == [b"a", b"b", b"c"]
    assert rt == b"abc"
    assert sp == [b"a", b"b", b"c"]
    assert sl == [b"a", b"b", b"c"]


def test_bytes_startswith_strip_swapcase_title_translate_upper_zfill() -> None:
    sw: bool = b"hello.py".startswith(b"he")
    st: bytes = b"...abc...".strip(b".")
    sc: bytes = b"AbC".swapcase()
    tt: bytes = b"hello world".title()

    table: bytes = b"_".maketrans(b"abc", b"xyz")
    tr: bytes = b"cab".translate(table)
    up: bytes = b"ab".upper()
    zf: bytes = b"42".zfill(5)

    print("bytes startswith:", sw)
    print("bytes strip:", st)
    print("bytes swapcase:", sc)
    print("bytes title:", tt)
    print("bytes translate:", tr)
    print("bytes upper:", up)
    print("bytes zfill:", zf)

    assert sw
    assert st == b"abc"
    assert sc == b"aBc"
    assert tt == b"Hello World"
    assert tr == b"zxy"
    assert up == b"AB"
    assert zf == b"00042"


def test_bytes_more_edges() -> None:
    # extra behavior checks
    c0: int = b"abc".count(b"")
    f0: int = b"abc".find(b"zz")
    rf0: int = b"abc".rfind(b"zz")
    rp0: bytes = b"abc".removeprefix(b"zz")
    rs0: bytes = b"abc".removesuffix(b"zz")
    zf_sign: bytes = b"-42".zfill(5)
    sl0: list[bytes] = b"".splitlines()

    print("bytes count empty sub:", c0)
    print("bytes find no match:", f0)
    print("bytes rfind no match:", rf0)
    print("bytes removeprefix no change:", rp0)
    print("bytes removesuffix no change:", rs0)
    print("bytes zfill sign:", zf_sign)
    print("bytes splitlines empty:", sl0)

    assert c0 == 4
    assert f0 == -1
    assert rf0 == -1
    assert rp0 == b"abc"
    assert rs0 == b"abc"
    assert zf_sign == b"-0042"
    assert sl0 == []


def test_bytes_complex_pipeline_transform() -> None:
    raw: bytes = b"\t  Alpha::BETA::gamma\t"
    step1: bytes = raw.expandtabs(2).strip(b" ")
    step2: bytes = step1.replace(b"::", b"|")
    parts: list[bytes] = step2.split(b"|")
    normalized: bytes = b"-".join(parts).lower().replace(b"\t", b"")

    print("bytes complex raw:", raw)
    print("bytes complex step1:", step1)
    print("bytes complex step2:", step2)
    print("bytes complex parts:", parts)
    print("bytes complex normalized:", normalized)

    assert len(parts) == 3
    assert parts[0].startswith(b"Alpha")
    assert parts[1] == b"BETA"
    assert parts[2].endswith(b"gamma")
    assert normalized == b"alpha-beta-gamma"


def test_bytes_complex_partition_roundtrip() -> None:
    data: bytes = b"left=middle=right"
    seps: list[bytes] = [b"=", b":", b"middle"]

    i: int = 0
    while i < len(seps):
        sep: bytes = seps[i]
        p: tuple[bytes, bytes, bytes] = data.partition(sep)
        rp: tuple[bytes, bytes, bytes] = data.rpartition(sep)

        joined_p: bytes = p[0] + p[1] + p[2]
        joined_rp: bytes = rp[0] + rp[1] + rp[2]

        print("bytes partition sep:", sep, "->", p[0], p[1], p[2])
        print("bytes rpartition sep:", sep, "->", rp[0], rp[1], rp[2])
        print("bytes partition joined:", joined_p)
        print("bytes rpartition joined:", joined_rp)

        assert joined_p == data
        assert joined_rp == data
        i += 1


def test_bytes_complex_hex_translate_roundtrip() -> None:
    payloads: list[bytes] = [b"", b"\x00\x01\x02", b"ABCxyz09", b"line1\nline2\r\n"]
    table: bytes = b"_".maketrans(
        b"abcdefghijklmnopqrstuvwxyz",
        b"ZYXWVUTSRQPONMLKJIHGFEDCBA",
    )

    i: int = 0
    while i < len(payloads):
        p: bytes = payloads[i]
        hx: str = p.hex()
        back: bytes = b"_".fromhex(hx)
        mapped: bytes = p.lower().translate(table)

        print("bytes payload:", p)
        print("bytes hex:", hx)
        print("bytes back:", back)
        print("bytes mapped:", mapped)

        assert back == p
        assert len(mapped) == len(p)
        i += 1


def test_bytes_complex_search_grid() -> None:
    hay: bytes = b"abbaabbaabba"
    needles: list[bytes] = [b"ab", b"ba", b"bb", b"zz"]

    i: int = 0
    while i < len(needles):
        n: bytes = needles[i]
        first: int = hay.find(n)
        last: int = hay.rfind(n)
        count: int = hay.count(n)

        print("bytes search needle:", n, "first:", first, "last:", last, "count:", count)

        if n == b"zz":
            assert first == -1
            assert last == -1
            assert count == 0
        else:
            assert first >= 0
            assert last >= first
            assert count >= 1
        i += 1


def run_tests() -> None:
    test_bytes_core_ops()
    test_bytes_capitalize_center_count_decode()
    test_bytes_endswith_expandtabs_find_fromhex_hex_index()
    test_bytes_predicates()
    test_bytes_join_ljust_lower_lstrip()
    test_bytes_maketrans_partition_prefix_suffix_replace()
    test_bytes_rfind_rindex_rjust_rpartition()
    test_bytes_rsplit_rstrip_split_splitlines()
    test_bytes_startswith_strip_swapcase_title_translate_upper_zfill()
    test_bytes_more_edges()
    test_bytes_complex_pipeline_transform()
    test_bytes_complex_partition_roundtrip()
    test_bytes_complex_hex_translate_roundtrip()
    test_bytes_complex_search_grid()
