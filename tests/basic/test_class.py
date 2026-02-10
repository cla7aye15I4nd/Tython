class Point:
    x: int
    y: int

    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

    def magnitude_sq(self) -> int:
        return self.x * self.x + self.y * self.y

    def sum(self) -> int:
        return self.x + self.y


class Counter:
    count: int

    def __init__(self, start: int) -> None:
        self.count = start

    def increment(self) -> None:
        self.count = self.count + 1

    def add(self, n: int) -> None:
        self.count = self.count + n

    def get(self) -> int:
        return self.count


class FloatBox:
    value: float

    def __init__(self, v: float) -> None:
        self.value = v

    def doubled(self) -> float:
        return self.value * 2.0


class BoolFlag:
    flag: bool

    def __init__(self, f: bool) -> None:
        self.flag = f

    def is_set(self) -> bool:
        return self.flag


class Container:
    value: int
    inner: Point

    def __init__(self, v: int, p: Point) -> None:
        self.value = v
        self.inner = p

    def inner_sum(self) -> int:
        return self.inner.x + self.inner.y

    def get_inner(self) -> Point:
        return self.inner


class PointFactory:
    ox: int
    oy: int

    def __init__(self, ox: int, oy: int) -> None:
        self.ox = ox
        self.oy = oy

    def make(self, x: int, y: int) -> Point:
        return Point(self.ox + x, self.oy + y)


def point_distance_sq(p1: Point, p2: Point) -> int:
    dx: int = p1.x - p2.x
    dy: int = p1.y - p2.y
    return dx * dx + dy * dy


# ── tests ─────────────────────────────────────────────────────────


def test_construct_and_fields() -> None:
    p: Point = Point(3, 4)
    print(p.x)
    assert p.x == 3
    print(p.y)
    assert p.y == 4


def test_method_return_value() -> None:
    p: Point = Point(3, 4)
    result: int = p.magnitude_sq()
    print(result)
    assert result == 25


def test_method_multiple() -> None:
    p: Point = Point(5, 7)
    print(p.sum())
    assert p.sum() == 12
    print(p.magnitude_sq())
    assert p.magnitude_sq() == 74


def test_field_mutation() -> None:
    p: Point = Point(1, 2)
    p.x = 10
    print(p.x)
    assert p.x == 10
    print(p.y)
    assert p.y == 2


def test_field_mutation_and_method() -> None:
    p: Point = Point(3, 4)
    p.x = 6
    p.y = 8
    result: int = p.magnitude_sq()
    print(result)
    assert result == 100


def test_stack_rebinding() -> None:
    p: Point = Point(3, 4)
    print(p.x)
    assert p.x == 3
    p = Point(5, 6)
    print(p.x)
    assert p.x == 5
    print(p.y)
    assert p.y == 6


def test_multiple_instances() -> None:
    a: Point = Point(1, 2)
    b: Point = Point(3, 4)
    print(a.x)
    assert a.x == 1
    print(b.x)
    assert b.x == 3
    a.x = 99
    print(a.x)
    assert a.x == 99
    print(b.x)
    assert b.x == 3


def test_void_method() -> None:
    c: Counter = Counter(0)
    c.increment()
    c.increment()
    c.increment()
    print(c.get())
    assert c.get() == 3


def test_void_method_with_arg() -> None:
    c: Counter = Counter(10)
    c.add(5)
    print(c.get())
    assert c.get() == 15
    c.add(100)
    print(c.get())
    assert c.get() == 115


def test_counter_loop() -> None:
    c: Counter = Counter(0)
    i: int = 0
    while i < 10:
        c.increment()
        i = i + 1
    print(c.get())
    assert c.get() == 10


def test_float_field() -> None:
    fb: FloatBox = FloatBox(3.14)
    print(fb.value)
    assert fb.value == 3.14
    print(fb.doubled())
    assert fb.doubled() == 6.28


def test_bool_field() -> None:
    f: BoolFlag = BoolFlag(True)
    print(f.is_set())
    assert f.is_set()
    g: BoolFlag = BoolFlag(False)
    print(g.is_set())
    assert not g.is_set()


def test_method_returns_instance() -> None:
    c: Container = Container(1, Point(10, 20))
    p: Point = c.get_inner()
    print(p.x)
    assert p.x == 10
    print(p.y)
    assert p.y == 20


def test_method_takes_instance_arg() -> None:
    f: PointFactory = PointFactory(100, 200)
    p: Point = f.make(5, 10)
    print(p.x)
    assert p.x == 105
    print(p.y)
    assert p.y == 210


def test_nested_class_fields() -> None:
    c: Container = Container(42, Point(10, 20))
    print(c.value)
    assert c.value == 42
    print(c.inner.x)
    assert c.inner.x == 10
    print(c.inner.y)
    assert c.inner.y == 20


def test_nested_class_method() -> None:
    c: Container = Container(1, Point(3, 7))
    print(c.inner_sum())
    assert c.inner_sum() == 10


def test_nested_field_mutation() -> None:
    c: Container = Container(1, Point(5, 6))
    c.value = 99
    print(c.value)
    assert c.value == 99
    c.inner.x = 50
    print(c.inner.x)
    assert c.inner.x == 50


def test_free_function_with_class_args() -> None:
    p1: Point = Point(0, 0)
    p2: Point = Point(3, 4)
    d: int = point_distance_sq(p1, p2)
    print(d)
    assert d == 25


def test_augmented_assign_field() -> None:
    c: Counter = Counter(10)
    c.count += 5
    print(c.count)
    assert c.count == 15
    c.count -= 3
    print(c.count)
    assert c.count == 12
    c.count *= 2
    print(c.count)
    assert c.count == 24


def test_field_in_expression() -> None:
    p: Point = Point(3, 4)
    result: int = p.x + p.y * 2
    print(result)
    assert result == 11


def test_field_as_condition() -> None:
    c: Counter = Counter(0)
    if c.count == 0:
        print(1)
    else:
        print(0)
    assert c.count == 0
    c.count = 5
    if c.count > 3:
        print(1)
    else:
        print(0)
    assert c.count > 3


def test_class_in_while_condition() -> None:
    c: Counter = Counter(0)
    while c.get() < 5:
        c.increment()
    print(c.get())
    assert c.get() == 5


def run_tests() -> None:
    test_construct_and_fields()
    test_method_return_value()
    test_method_multiple()
    test_field_mutation()
    test_field_mutation_and_method()
    test_stack_rebinding()
    test_multiple_instances()
    test_void_method()
    test_void_method_with_arg()
    test_counter_loop()
    test_float_field()
    test_bool_field()
    test_method_returns_instance()
    test_method_takes_instance_arg()
    test_nested_class_fields()
    test_nested_class_method()
    test_nested_field_mutation()
    test_free_function_with_class_args()
    test_augmented_assign_field()
    test_field_in_expression()
    test_field_as_condition()
    test_class_in_while_condition()
