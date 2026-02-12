def test_bytearray_eq() -> None:
    a: bytearray = bytearray(b"hello")
    b: bytearray = bytearray(b"hello")
    assert a == b
    print("ba_eq ok")

def test_bytearray_neq() -> None:
    a: bytearray = bytearray(b"hello")
    b: bytearray = bytearray(b"world")
    assert a != b
    print("ba_neq ok")

def test_bytearray_lt() -> None:
    a: bytearray = bytearray(b"abc")
    b: bytearray = bytearray(b"abd")
    assert a < b
    print("ba_lt ok")

def test_bytearray_gt() -> None:
    a: bytearray = bytearray(b"xyz")
    b: bytearray = bytearray(b"abc")
    assert a > b
    print("ba_gt ok")

def test_bytearray_concat() -> None:
    a: bytearray = bytearray(b"hello")
    b: bytearray = bytearray(b" world")
    c: bytearray = a + b
    assert len(c) == 11
    print("ba_concat ok")

def test_bytearray_repeat() -> None:
    a: bytearray = bytearray(b"ab")
    b: bytearray = a * 3
    assert len(b) == 6
    print("ba_repeat ok")

def run_tests() -> None:
    test_bytearray_eq()
    test_bytearray_neq()
    test_bytearray_lt()
    test_bytearray_gt()
    test_bytearray_concat()
    test_bytearray_repeat()
