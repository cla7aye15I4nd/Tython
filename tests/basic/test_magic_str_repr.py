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


def run_tests() -> None:
    test_str_uses_dunder_str()
    test_repr_uses_dunder_repr()
    test_print_uses_str()
    test_str_falls_back_to_repr()
