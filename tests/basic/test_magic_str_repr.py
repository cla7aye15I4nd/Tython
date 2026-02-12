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


def test_str_uses_dunder_str() -> None:
    p: Person = Person("alice")
    s: str = str(p)
    print(s)
    assert s == "Person(alice)"


def test_repr_uses_dunder_repr() -> None:
    p: Person = Person("bob")
    r: str = repr(p)
    print(r)
    assert r == "Person<bob>"


def test_print_uses_str() -> None:
    p: Person = Person("carol")
    print(p)


def test_str_falls_back_to_repr() -> None:
    x: OnlyRepr = OnlyRepr(7)
    s: str = str(x)
    print(s)
    assert s == "OnlyRepr#7"
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


def run_tests() -> None:
    test_str_uses_dunder_str()
    test_repr_uses_dunder_repr()
    test_print_uses_str()
    test_str_falls_back_to_repr()
    test_print_tuple_and_list_of_class()
    test_print_recursive_nested_values()
