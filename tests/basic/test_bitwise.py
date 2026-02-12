def test_bitand() -> None:
    x: int = 12 & 10
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 8)
    assert x == 8


def test_bitor() -> None:
    x: int = 12 | 10
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 14)
    assert x == 14


def test_bitxor() -> None:
    x: int = 12 ^ 10
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 6)
    assert x == 6


def test_lshift() -> None:
    x: int = 1 << 4
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 16)
    assert x == 16


def test_rshift() -> None:
    x: int = 16 >> 2
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 4)
    assert x == 4


def test_rshift_negative() -> None:
    x: int = -8 >> 1
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', -4)
    assert x == -4


def test_bitand_mask() -> None:
    x: int = 255 & 15
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 15)
    assert x == 15


def test_bitor_combine() -> None:
    x: int = 3 | 12
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 15)
    assert x == 15


def test_bitxor_toggle() -> None:
    x: int = 15 ^ 15
    print('CHECK test_bitwise lhs:', x)
    print('CHECK test_bitwise rhs:', 0)
    assert x == 0


def run_tests() -> None:
    test_bitand()
    test_bitor()
    test_bitxor()
    test_lshift()
    test_rshift()
    test_rshift_negative()
    test_bitand_mask()
    test_bitor_combine()
    test_bitxor_toggle()
