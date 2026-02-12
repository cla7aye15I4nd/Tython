class NumberBox:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __neg__(self) -> int:
        return 0 - self.value

    def __pos__(self) -> int:
        return self.value

    def __invert__(self) -> int:
        return ~self.value

    def __bool__(self) -> bool:
        return self.value != 0

    def __add__(self, rhs: int) -> int:
        return self.value + rhs

    def __radd__(self, lhs: int) -> int:
        return lhs + self.value

    def __sub__(self, rhs: int) -> int:
        return self.value - rhs

    def __rsub__(self, lhs: int) -> int:
        return lhs - self.value

    def __mul__(self, rhs: int) -> int:
        return self.value * rhs

    def __rmul__(self, lhs: int) -> int:
        return lhs * self.value

    def __truediv__(self, rhs: int) -> int:
        return self.value // rhs

    def __rtruediv__(self, lhs: int) -> int:
        return lhs // self.value

    def __abs__(self) -> int:
        if self.value < 0:
            return 0 - self.value
        return self.value

    def __round__(self) -> int:
        return self.value + 1

    def __int__(self) -> int:
        return self.value

    def __float__(self) -> float:
        return float(self.value)

    def __bytes__(self) -> bytes:
        return bytes(self.value)


def test_unary_magic_on_class() -> None:
    a: NumberBox = NumberBox(7)
    b: NumberBox = NumberBox(0)

    print('CHECK test_magic_ops lhs:', -a)
    print('CHECK test_magic_ops rhs:', -7)
    assert -a == -7

    print('CHECK test_magic_ops lhs:', +a)
    print('CHECK test_magic_ops rhs:', 7)
    assert +a == 7

    print('CHECK test_magic_ops lhs:', ~a)
    print('CHECK test_magic_ops rhs:', ~7)
    assert ~a == ~7

    print('CHECK test_magic_ops lhs:', not a)
    print('CHECK test_magic_ops rhs:', False)
    assert (not a) == False

    print('CHECK test_magic_ops lhs:', not b)
    print('CHECK test_magic_ops rhs:', True)
    assert (not b) == True


def test_binop_magic_and_reverse() -> None:
    x: NumberBox = NumberBox(10)

    print('CHECK test_magic_ops lhs:', x + 3)
    print('CHECK test_magic_ops rhs:', 13)
    assert x + 3 == 13

    print('CHECK test_magic_ops lhs:', 3 + x)
    print('CHECK test_magic_ops rhs:', 13)
    assert 3 + x == 13

    print('CHECK test_magic_ops lhs:', x - 4)
    print('CHECK test_magic_ops rhs:', 6)
    assert x - 4 == 6

    print('CHECK test_magic_ops lhs:', 25 - x)
    print('CHECK test_magic_ops rhs:', 15)
    assert 25 - x == 15

    print('CHECK test_magic_ops lhs:', x * 2)
    print('CHECK test_magic_ops rhs:', 20)
    assert x * 2 == 20

    print('CHECK test_magic_ops lhs:', 2 * x)
    print('CHECK test_magic_ops rhs:', 20)
    assert 2 * x == 20

    print('CHECK test_magic_ops lhs:', x / 2)
    print('CHECK test_magic_ops rhs:', 5)
    assert x / 2 == 5

    print('CHECK test_magic_ops lhs:', 40 / x)
    print('CHECK test_magic_ops rhs:', 4)
    assert 40 / x == 4


def test_builtin_magic_on_class() -> None:
    p: NumberBox = NumberBox(-5)
    q: NumberBox = NumberBox(4)

    print('CHECK test_magic_ops lhs:', abs(p))
    print('CHECK test_magic_ops rhs:', 5)
    assert abs(p) == 5

    print('CHECK test_magic_ops lhs:', round(q))
    print('CHECK test_magic_ops rhs:', 5)
    assert round(q) == 5

    print('CHECK test_magic_ops lhs:', int(q))
    print('CHECK test_magic_ops rhs:', 4)
    assert int(q) == 4

    print('CHECK test_magic_ops lhs:', float(q))
    print('CHECK test_magic_ops rhs:', 4.0)
    assert float(q) == 4.0

    b: bytes = bytes(q)
    print('CHECK test_magic_ops lhs:', len(b))
    print('CHECK test_magic_ops rhs:', 4)
    assert len(b) == 4

    print('CHECK test_magic_ops lhs:', bool(q))
    print('CHECK test_magic_ops rhs:', True)
    assert bool(q) == True


def run_tests() -> None:
    test_unary_magic_on_class()
    test_binop_magic_and_reverse()
    test_builtin_magic_on_class()
