class Vec2:
    x: int
    y: int

    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

    def __add__(self, rhs: int) -> int:
        return self.x + self.y + rhs

    def __radd__(self, lhs: int) -> int:
        return lhs + self.x + self.y

    def __sub__(self, rhs: int) -> int:
        return self.x + self.y - rhs

    def __rsub__(self, lhs: int) -> int:
        return lhs - (self.x + self.y)

    def __mul__(self, rhs: int) -> int:
        return (self.x + self.y) * rhs

    def __rmul__(self, lhs: int) -> int:
        return lhs * (self.x + self.y)

    def __neg__(self) -> int:
        return 0 - (self.x + self.y)

    def __abs__(self) -> int:
        s: int = self.x + self.y
        if s < 0:
            return 0 - s
        return s

    def __bool__(self) -> bool:
        return (self.x + self.y) != 0


def fold_values(base: int, vs: list[Vec2]) -> int:
    acc: int = base
    for v in vs:
        acc = acc + v
        acc = acc - v
        acc = acc + (2 * v)
    return acc


def test_complex_reverse_dispatch_chain() -> None:
    xs: list[Vec2] = [Vec2(1, 2), Vec2(3, 4), Vec2(-2, 1)]
    out: int = fold_values(5, xs)

    print('CHECK test_magic_ops_complex lhs:', out)
    print('CHECK test_magic_ops_complex rhs:', 23)
    assert out == 23


def test_complex_unary_builtin_dispatch_chain() -> None:
    a: Vec2 = Vec2(-3, 1)
    b: Vec2 = Vec2(0, 0)

    n: int = -a
    m: int = abs(a)

    print('CHECK test_magic_ops_complex lhs:', n)
    print('CHECK test_magic_ops_complex rhs:', 2)
    assert n == 2

    print('CHECK test_magic_ops_complex lhs:', m)
    print('CHECK test_magic_ops_complex rhs:', 2)
    assert m == 2

    print('CHECK test_magic_ops_complex lhs:', not a)
    print('CHECK test_magic_ops_complex rhs:', False)
    assert (not a) == False

    print('CHECK test_magic_ops_complex lhs:', not b)
    print('CHECK test_magic_ops_complex rhs:', True)
    assert (not b) == True


def run_tests() -> None:
    test_complex_reverse_dispatch_chain()
    test_complex_unary_builtin_dispatch_chain()
