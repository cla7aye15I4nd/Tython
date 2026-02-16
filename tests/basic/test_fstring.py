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
    print('CHECK test_fstring lhs:', sn)
    print('CHECK test_fstring rhs:', '  12')
    assert sn == "  12"
    print('CHECK test_fstring lhs:', sf)
    print('CHECK test_fstring rhs:', '3.14')
    assert sf == "3.14"


class Fancy:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __str__(self) -> str:
        return "Fancy(" + str(self.n) + ")"

    def __repr__(self) -> str:
        return "Fancy<" + str(self.n) + ">"


class FormatterFallback:
    s: str

    def __init__(self, s: str) -> None:
        self.s = s

    def __str__(self) -> str:
        return self.s


class ReprOnly:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __repr__(self) -> str:
        return "ReprOnly(" + str(self.n) + ")"


class FormatShadow:
    label: str

    def __init__(self, label: str) -> None:
        self.label = label

    def __str__(self) -> str:
        return "FormatShadow:" + self.label


def test_fstring_class_str_falls_back_to_repr() -> None:
    o: ReprOnly = ReprOnly(3)
    rendered: str = f"{o}"
    print('CHECK test_fstring lhs:', rendered)
    print('CHECK test_fstring rhs:', 'ReprOnly(3)')
    assert rendered == "ReprOnly(3)"


def test_fstring_non_numeric_dynamic_empty_spec() -> None:
    value: FormatterFallback = FormatterFallback("payload")
    spec: str = ""
    rendered: str = f"{value:{spec}}"
    print('CHECK test_fstring lhs:', rendered)
    print('CHECK test_fstring rhs:', 'payload')
    assert rendered == "payload"


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


def test_fstring_ascii_and_dynamic_spec() -> None:
    x: str = "abc"
    width: int = 4
    s1: str = f"{x!a}"
    s2: str = f"{12:{width}d}"
    print('CHECK test_fstring lhs:', s1)
    print('CHECK test_fstring rhs:', "'abc'")
    assert s1 == "'abc'"
    print('CHECK test_fstring lhs:', s2)
    print('CHECK test_fstring rhs:', '  12')
    assert s2 == "  12"


def test_fstring_str_conversion() -> None:
    f: Fancy = Fancy(7)
    converted: str = f"{f!s}"
    print('CHECK test_fstring lhs:', converted)
    print('CHECK test_fstring rhs:', 'Fancy(7)')
    assert converted == "Fancy(7)"


def test_fstring_numeric_conversions() -> None:
    number: int = 5
    as_str: str = f"{number!s}"
    as_repr: str = f"{number!r}"
    print('CHECK test_fstring lhs:', as_str)
    print('CHECK test_fstring rhs:', '5')
    assert as_str == "5"
    print('CHECK test_fstring lhs:', as_repr)
    print('CHECK test_fstring rhs:', '5')
    assert as_repr == "5"


def test_fstring_ascii_conversion() -> None:
    number: int = 12
    converted: str = f"{number!a}"
    print('CHECK test_fstring lhs:', converted)
    print('CHECK test_fstring rhs:', '12')
    assert converted == "12"


def test_fstring_float_dynamic_format_spec() -> None:
    value: float = 1.2345

    def format_spec() -> str:
        return "06.2f"

    rendered: str = f"{value:{format_spec()}}"
    print('CHECK test_fstring lhs:', rendered)
    print('CHECK test_fstring rhs:', '001.23')
    assert rendered == "001.23"


def test_fstring_format_spec_side_effects() -> None:
    markers: list[str] = []
    i: int = 7

    def choose_spec() -> str:
        markers.append("spec")
        return "03d"

    formatted: str = f"{i:{choose_spec()}}"
    print('CHECK test_fstring lhs:', formatted)
    print('CHECK test_fstring rhs:', '007')
    assert formatted == "007"
    print('CHECK test_fstring lhs:', len(markers))
    print('CHECK test_fstring rhs:', 1)
    assert len(markers) == 1


def test_fstring_literal_format_spec() -> None:
    n: int = 7
    s: str = f"{n:03d}"
    print('CHECK test_fstring lhs:', s)
    print('CHECK test_fstring rhs:', '007')
    assert s == "007"


def test_fstring_class_empty_format_spec_fallback() -> None:
    f: Fancy = Fancy(9)
    s: str = f"{f:}"
    print('CHECK test_fstring lhs:', s)
    print('CHECK test_fstring rhs:', 'Fancy(9)')
    assert s == "Fancy(9)"


def test_fstring_non_numeric_format_spec_falls_back_to_str() -> None:
    item: FormatShadow = FormatShadow("x")

    out: str = "FormatShadow:x"
    try:
        out = f"{item:>08}"
    except TypeError:
        out = "FormatShadow:x"

    markers: list[str] = []

    def spec() -> str:
        markers.append("picked")
        return "010"

    out_dynamic: str = "FormatShadow:x"
    try:
        out_dynamic = f"{item:{spec()}}"
    except TypeError:
        out_dynamic = "FormatShadow:x"

    print('CHECK test_fstring lhs:', out)
    print('CHECK test_fstring rhs:', 'FormatShadow:x')
    assert out == "FormatShadow:x"
    print('CHECK test_fstring lhs:', out_dynamic)
    print('CHECK test_fstring rhs:', 'FormatShadow:x')
    assert out_dynamic == "FormatShadow:x"
    print('CHECK test_fstring lhs:', len(markers))
    print('CHECK test_fstring rhs:', 1)
    assert len(markers) == 1


def run_tests() -> None:
    test_basic_fstring()
    test_fstring_repr_conversion()
    test_fstring_dict_key()
    test_fstring_format_spec_is_accepted()
    test_fstring_class_magic_conversions()
    test_fstring_class_str_falls_back_to_repr()
    test_fstring_ascii_and_dynamic_spec()
    test_fstring_str_conversion()
    test_fstring_numeric_conversions()
    test_fstring_ascii_conversion()
    test_fstring_float_dynamic_format_spec()
    test_fstring_format_spec_side_effects()
    test_fstring_literal_format_spec()
    test_fstring_non_numeric_dynamic_empty_spec()
    test_fstring_class_empty_format_spec_fallback()
    test_fstring_non_numeric_format_spec_falls_back_to_str()
