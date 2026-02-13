def test_bytearray_construction_and_core_ops() -> None:
    a: bytearray = bytearray()
    assert len(a) == 0

    b: bytearray = bytearray(3)
    assert len(b) == 3

    c: bytearray = bytearray(b"abc")
    assert len(c) == 3

    d: bytearray = c + bytearray(b"de")
    assert len(d) == 5

    e: bytearray = c * 2
    assert len(e) == 6


def test_bytearray_mutation_methods() -> None:
    ba: bytearray = bytearray(b"ab")

    ba.append(99)
    assert len(ba) == 3

    ba.extend(b"de")
    assert len(ba) == 5

    ba.insert(1, 120)
    assert len(ba) == 6

    ba.remove(120)
    assert len(ba) == 5

    ba.reverse()
    assert len(ba) == 5

    ba.clear()
    assert len(ba) == 0


def test_bytearray_misc_methods_group1() -> None:
    cap: bytearray = bytearray(b"hELLo").capitalize()
    cen: bytearray = bytearray(b"x").center(5, b"-")
    cnt: int = bytearray(b"aaaa").count(b"aa")
    dec: str = bytearray(b"abc").decode()
    ew: bool = bytearray(b"alpha.py").endswith(b".py")
    ex: bytearray = bytearray(b"a\tb").expandtabs(4)
    fd: int = bytearray(b"banana").find(b"na")
    fh: bytearray = bytearray(b"ignored").fromhex("61 62 63")
    hx: str = bytearray(b"abc").hex()
    ix: int = bytearray(b"banana").index(b"na")

    assert cap == bytearray(b"Hello")
    assert cen == bytearray(b"--x--")
    assert cnt == 2
    assert dec == "abc"
    assert ew
    assert ex == bytearray(b"a   b")
    assert fd == 2
    assert fh == bytearray(b"abc")
    assert hx == "616263"
    assert ix == 2


def test_bytearray_predicates() -> None:
    assert bytearray(b"abc123").isalnum()
    assert bytearray(b"abc").isalpha()
    assert bytearray(b"abc").isascii()
    assert not bytearray(b"\xff").isascii()
    assert bytearray(b"123").isdigit()
    assert bytearray(b"abc").islower()
    assert bytearray(b" \t\n").isspace()
    assert bytearray(b"Hello World").istitle()
    assert bytearray(b"ABC").isupper()


def test_bytearray_misc_methods_group2() -> None:
    parts: list[bytearray] = [bytearray(b"a"), bytearray(b"b"), bytearray(b"c")]
    jn: bytearray = bytearray(b"-").join(parts)
    lj: bytearray = bytearray(b"x").ljust(4, b".")
    lo: bytearray = bytearray(b"AbC").lower()
    ls: bytearray = bytearray(b"000123").lstrip(b"0")
    table: bytes = bytearray(b"_").maketrans(b"abc", b"xyz")
    p: tuple[bytearray, bytearray, bytearray] = bytearray(b"key=value").partition(b"=")
    rp: bytearray = bytearray(b"prefix_data").removeprefix(b"prefix_")
    rs: bytearray = bytearray(b"data.txt").removesuffix(b".txt")
    rr: bytearray = bytearray(b"one one").replace(b"one", b"two")
    rf: int = bytearray(b"banana").rfind(b"na")
    ri: int = bytearray(b"banana").rindex(b"na")
    rj: bytearray = bytearray(b"x").rjust(4, b".")
    rp2: tuple[bytearray, bytearray, bytearray] = bytearray(b"a=b=c").rpartition(b"=")
    rspl: list[bytearray] = bytearray(b"a,b,c").rsplit(b",")
    rst: bytearray = bytearray(b"abc...").rstrip(b".")
    spl: list[bytearray] = bytearray(b"a,b,c").split(b",")
    sll: list[bytearray] = bytearray(b"a\nb\r\nc").splitlines()
    sw: bool = bytearray(b"hello.py").startswith(b"he")
    st: bytearray = bytearray(b"...abc...").strip(b".")
    sc: bytearray = bytearray(b"AbC").swapcase()
    tt: bytearray = bytearray(b"hello world").title()
    tr: bytearray = bytearray(b"cab").translate(table)
    up: bytearray = bytearray(b"ab").upper()
    zf: bytearray = bytearray(b"42").zfill(5)

    assert jn == bytearray(b"a-b-c")
    assert lj == bytearray(b"x...")
    assert lo == bytearray(b"abc")
    assert ls == bytearray(b"123")
    assert len(table) == 256
    assert p[0] == bytearray(b"key")
    assert p[1] == bytearray(b"=")
    assert p[2] == bytearray(b"value")
    assert rp == bytearray(b"data")
    assert rs == bytearray(b"data")
    assert rr == bytearray(b"two two")
    assert rf == 4
    assert ri == 4
    assert rj == bytearray(b"...x")
    assert rp2[0] == bytearray(b"a=b")
    assert rp2[1] == bytearray(b"=")
    assert rp2[2] == bytearray(b"c")
    assert rspl == [bytearray(b"a"), bytearray(b"b"), bytearray(b"c")]
    assert rst == bytearray(b"abc")
    assert spl == [bytearray(b"a"), bytearray(b"b"), bytearray(b"c")]
    assert sll == [bytearray(b"a"), bytearray(b"b"), bytearray(b"c")]
    assert sw
    assert st == bytearray(b"abc")
    assert sc == bytearray(b"aBc")
    assert tt == bytearray(b"Hello World")
    assert tr == bytearray(b"zxy")
    assert up == bytearray(b"AB")
    assert zf == bytearray(b"00042")


def test_bytearray_copy_and_pop() -> None:
    ba: bytearray = bytearray(b"xyz")
    cp: bytearray = ba.copy()
    assert cp == ba

    popped: int = ba.pop()
    assert popped == 122
    assert ba == bytearray(b"xy")


def test_bytearray_compare_truthiness_and_str() -> None:
    x: bytearray = bytearray(b"abc")
    y: bytearray = bytearray(b"abc")
    z: bytearray = bytearray(b"abd")

    assert x == y
    assert x != z
    assert x < z

    assert x
    assert not bytearray()

    rep: str = str(x)
    assert "bytearray(" in rep


def test_bytearray_alias_uses_bytearray_api() -> None:
    ba: bytearray = bytearray(b"xy")
    ba.append(122)
    ba.extend(b"!")
    assert len(ba) == 4
    assert ba == bytearray(b"xyz!")


def run_tests() -> None:
    test_bytearray_construction_and_core_ops()
    test_bytearray_mutation_methods()
    test_bytearray_misc_methods_group1()
    test_bytearray_predicates()
    test_bytearray_misc_methods_group2()
    test_bytearray_copy_and_pop()
    test_bytearray_compare_truthiness_and_str()
    test_bytearray_alias_uses_bytearray_api()
