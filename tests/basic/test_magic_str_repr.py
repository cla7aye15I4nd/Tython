class Person:
    name: str

    def __init__(self, name: str) -> None:
        self.name = name

    def __str__(self) -> str:
        return "Person(" + self.name + ")"

    def __repr__(self) -> str:
        return "Person<" + self.name + ">"


class OnlyRepr:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __repr__(self) -> str:
        return "OnlyRepr#" + str(self.value)


class NumericMagic:
    v: int

    def __init__(self, v: int) -> None:
        self.v = v

    def __abs__(self) -> int:
        if self.v < 0:
            return 0 - self.v
        return self.v

    def __round__(self) -> int:
        return self.v

    def __int__(self) -> int:
        return self.v

    def __float__(self) -> float:
        return float(self.v)

    def __bool__(self) -> bool:
        return self.v != 0


class BytesMagic:
    payload: bytes

    def __init__(self, payload: bytes) -> None:
        self.payload = payload

    def __bytes__(self) -> bytes:
        return self.payload


def test_str_uses_dunder_str() -> None:
    p: Person = Person("alice")
    s: str = str(p)
    print('CHECK test_magic_str_repr lhs:', s)
    print('CHECK test_magic_str_repr rhs:', 'Person(alice)')
    assert s == "Person(alice)"


def test_repr_uses_dunder_repr() -> None:
    p: Person = Person("bob")
    r: str = repr(p)
    print('CHECK test_magic_str_repr lhs:', r)
    print('CHECK test_magic_str_repr rhs:', 'Person<bob>')
    assert r == "Person<bob>"


def test_print_uses_str() -> None:
    p: Person = Person("carol")
    print(p)


def test_str_falls_back_to_repr() -> None:
    x: OnlyRepr = OnlyRepr(7)
    s: str = str(x)
    print(s)
    print('CHECK test_magic_str_repr assert expr:', 's == "OnlyRepr')
    assert s == "OnlyRepr#7"
    print('CHECK test_magic_str_repr assert expr:', 'repr(x) == "OnlyRepr')
    assert repr(x) == "OnlyRepr#7"


def test_print_tuple_and_list_of_class() -> None:
    a: OnlyRepr = OnlyRepr(3)
    b: OnlyRepr = OnlyRepr(4)
    pair: tuple[OnlyRepr, OnlyRepr] = (a, b)
    single: tuple[OnlyRepr] = (a,)
    items: list[OnlyRepr] = [a, b]
    nested: tuple[list[OnlyRepr], OnlyRepr] = (items, b)

    print(pair)
    print(single)
    print(items)
    print(nested)


def test_print_recursive_nested_values() -> None:
    a: OnlyRepr = OnlyRepr(1)
    b: OnlyRepr = OnlyRepr(2)
    c: OnlyRepr = OnlyRepr(3)

    nested_lists: list[list[OnlyRepr]] = [[a, b], [c]]
    tuple_items: list[tuple[OnlyRepr, OnlyRepr]] = [(a, b), (b, c)]
    deep_tuple: tuple[list[list[OnlyRepr]], list[tuple[OnlyRepr, OnlyRepr]]] = (
        nested_lists,
        tuple_items,
    )

    print(nested_lists)
    print(tuple_items)
    print(deep_tuple)


def test_numeric_magic_builtins() -> None:
    x: NumericMagic = NumericMagic(-7)
    y: NumericMagic = NumericMagic(0)

    print('CHECK test_magic_str_repr lhs:', abs(x))
    print('CHECK test_magic_str_repr rhs:', 7)
    assert abs(x) == 7

    print('CHECK test_magic_str_repr lhs:', round(x))
    print('CHECK test_magic_str_repr rhs:', -7)
    assert round(x) == -7

    print('CHECK test_magic_str_repr lhs:', int(x))
    print('CHECK test_magic_str_repr rhs:', -7)
    assert int(x) == -7

    print('CHECK test_magic_str_repr lhs:', float(x))
    print('CHECK test_magic_str_repr rhs:', -7.0)
    assert float(x) == -7.0

    print('CHECK test_magic_str_repr lhs:', bool(x))
    print('CHECK test_magic_str_repr rhs:', True)
    assert bool(x) == True

    print('CHECK test_magic_str_repr lhs:', bool(y))
    print('CHECK test_magic_str_repr rhs:', False)
    assert bool(y) == False


def test_bytes_magic_builtin() -> None:
    b: BytesMagic = BytesMagic(b"xyz")
    out: bytes = bytes(b)
    print('CHECK test_magic_str_repr lhs:', out)
    print('CHECK test_magic_str_repr rhs:', b'xyz')
    assert out == b"xyz"


def run_tests() -> None:
    test_str_uses_dunder_str()
    test_repr_uses_dunder_repr()
    test_print_uses_str()
    test_str_falls_back_to_repr()
    test_print_tuple_and_list_of_class()
    test_print_recursive_nested_values()
    test_numeric_magic_builtins()
    test_bytes_magic_builtin()
