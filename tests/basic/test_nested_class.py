class Outer:
    value: int

    class Inner:
        x: int

        def __init__(self, x: int) -> None:
            self.x = x

        def get(self) -> int:
            return self.x

    def __init__(self, v: int) -> None:
        self.value = v

    def get_value(self) -> int:
        return self.value


class Deep:
    tag: int

    class Mid:
        mid_val: int

        class Leaf:
            leaf_val: int

            def __init__(self, v: int) -> None:
                self.leaf_val = v

            def get(self) -> int:
                return self.leaf_val

        def __init__(self, v: int) -> None:
            self.mid_val = v

        def get(self) -> int:
            return self.mid_val

    def __init__(self, t: int) -> None:
        self.tag = t


# ── tests ─────────────────────────────────────────────────────────


def test_nested_class_construct() -> None:
    inner: Outer.Inner = Outer.Inner(42)
    print(inner.get())
    print('CHECK test_nested_class lhs expr:', 'inner.get()')
    print('CHECK test_nested_class rhs:', 42)
    assert inner.get() == 42


def test_nested_class_field() -> None:
    inner: Outer.Inner = Outer.Inner(7)
    print(inner.x)
    print('CHECK test_nested_class lhs:', inner.x)
    print('CHECK test_nested_class rhs:', 7)
    assert inner.x == 7
    inner.x = 99
    print(inner.x)
    print('CHECK test_nested_class lhs:', inner.x)
    print('CHECK test_nested_class rhs:', 99)
    assert inner.x == 99


def test_outer_and_inner() -> None:
    o: Outer = Outer(10)
    i: Outer.Inner = Outer.Inner(20)
    print(o.get_value())
    print('CHECK test_nested_class lhs expr:', 'o.get_value()')
    print('CHECK test_nested_class rhs:', 10)
    assert o.get_value() == 10
    print(i.get())
    print('CHECK test_nested_class lhs expr:', 'i.get()')
    print('CHECK test_nested_class rhs:', 20)
    assert i.get() == 20


def test_deeply_nested() -> None:
    leaf: Deep.Mid.Leaf = Deep.Mid.Leaf(100)
    print(leaf.get())
    print('CHECK test_nested_class lhs expr:', 'leaf.get()')
    print('CHECK test_nested_class rhs:', 100)
    assert leaf.get() == 100
    mid: Deep.Mid = Deep.Mid(50)
    print(mid.get())
    print('CHECK test_nested_class lhs expr:', 'mid.get()')
    print('CHECK test_nested_class rhs:', 50)
    assert mid.get() == 50
    d: Deep = Deep(1)
    print(d.tag)
    print('CHECK test_nested_class lhs:', d.tag)
    print('CHECK test_nested_class rhs:', 1)
    assert d.tag == 1


def test_class_in_function() -> None:
    class Local:
        val: int

        def __init__(self, v: int) -> None:
            self.val = v

        def doubled(self) -> int:
            return self.val * 2

    a: Local = Local(5)
    print(a.doubled())
    print('CHECK test_nested_class lhs expr:', 'a.doubled()')
    print('CHECK test_nested_class rhs:', 10)
    assert a.doubled() == 10
    b: Local = Local(20)
    print(b.val)
    print('CHECK test_nested_class lhs:', b.val)
    print('CHECK test_nested_class rhs:', 20)
    assert b.val == 20


def test_class_in_function_with_nested() -> None:
    class Wrapper:
        n: int

        class Tag:
            t: int

            def __init__(self, t: int) -> None:
                self.t = t

            def get(self) -> int:
                return self.t

        def __init__(self, n: int) -> None:
            self.n = n

        def get_n(self) -> int:
            return self.n

    w: Wrapper = Wrapper(7)
    print(w.get_n())
    print('CHECK test_nested_class lhs expr:', 'w.get_n()')
    print('CHECK test_nested_class rhs:', 7)
    assert w.get_n() == 7
    tag: Wrapper.Tag = Wrapper.Tag(99)
    print(tag.get())
    print('CHECK test_nested_class lhs expr:', 'tag.get()')
    print('CHECK test_nested_class rhs:', 99)
    assert tag.get() == 99


def test_multiple_local_classes() -> None:
    class Alpha:
        a: int

        def __init__(self, a: int) -> None:
            self.a = a

        def get(self) -> int:
            return self.a

    class Beta:
        b: int

        def __init__(self, b: int) -> None:
            self.b = b

        def get(self) -> int:
            return self.b

    x: Alpha = Alpha(10)
    y: Beta = Beta(20)
    result: int = x.get() + y.get()
    print(result)
    print('CHECK test_nested_class lhs:', result)
    print('CHECK test_nested_class rhs:', 30)
    assert result == 30


def run_tests() -> None:
    test_nested_class_construct()
    test_nested_class_field()
    test_outer_and_inner()
    test_deeply_nested()
    test_class_in_function()
    test_class_in_function_with_nested()
    test_multiple_local_classes()
