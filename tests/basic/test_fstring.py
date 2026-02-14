def test_basic_fstring() -> None:
    who: str = "world"
    n: int = 7
    s: str = f"hello {who} {n}"
    print('CHECK test_fstring lhs:', s)
    print('CHECK test_fstring rhs:', 'hello world 7')
    assert s == "hello world 7"


def test_fstring_repr_conversion() -> None:
    s: str = f"{'x'!r}"
    print('CHECK test_fstring lhs:', s)
    print('CHECK test_fstring rhs:', "'x'")
    assert s == "'x'"


def test_fstring_dict_key() -> None:
    i: int = 3
    key: str = f"layer{i}.attn_wq"
    print('CHECK test_fstring lhs:', key)
    print('CHECK test_fstring rhs:', 'layer3.attn_wq')
    assert key == "layer3.attn_wq"


def test_fstring_format_spec_is_accepted() -> None:
    n: int = 12
    f: float = 3.14159
    sn: str = f"{n:4d}"
    sf: str = f"{f:.2f}"
    assert "12" in sn
    assert "3" in sf


class Fancy:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __str__(self) -> str:
        return "Fancy(" + str(self.n) + ")"

    def __repr__(self) -> str:
        return "Fancy<" + str(self.n) + ">"


def test_fstring_class_magic_conversions() -> None:
    f: Fancy = Fancy(9)
    s1: str = f"{f}"
    s2: str = f"{f!r}"
    print('CHECK test_fstring lhs:', s1)
    print('CHECK test_fstring rhs:', 'Fancy(9)')
    assert s1 == "Fancy(9)"
    print('CHECK test_fstring lhs:', s2)
    print('CHECK test_fstring rhs:', 'Fancy<9>')
    assert s2 == "Fancy<9>"


def run_tests() -> None:
    test_basic_fstring()
    test_fstring_repr_conversion()
    test_fstring_dict_key()
    test_fstring_format_spec_is_accepted()
    test_fstring_class_magic_conversions()
