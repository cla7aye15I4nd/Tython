class MagicNum:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __int__(self) -> int:
        return self.value

    def __float__(self) -> float:
        return float(self.value)

    def __bool__(self) -> bool:
        return self.value != 0

    def __len__(self) -> int:
        return self.value

    def __abs__(self) -> int:
        if self.value < 0:
            return 0 - self.value
        return self.value

    def __round__(self) -> int:
        return self.value

    def __bytes__(self) -> bytes:
        return bytes(self.value)

    def __iter__(self) -> "MagicNum":
        return self

    def __next__(self) -> int:
        return self.value


def test_builtin_magic_dispatch_paths() -> None:
    m: MagicNum = MagicNum(5)

    print('CHECK test_builtin_magic_dispatch lhs:', int(m))
    print('CHECK test_builtin_magic_dispatch rhs:', 5)
    assert int(m) == 5

    print('CHECK test_builtin_magic_dispatch lhs:', float(m))
    print('CHECK test_builtin_magic_dispatch rhs:', 5.0)
    assert float(m) == 5.0

    print('CHECK test_builtin_magic_dispatch lhs:', bool(m))
    print('CHECK test_builtin_magic_dispatch rhs:', True)
    assert bool(m) == True

    print('CHECK test_builtin_magic_dispatch lhs:', len(m))
    print('CHECK test_builtin_magic_dispatch rhs:', 5)
    assert len(m) == 5

    print('CHECK test_builtin_magic_dispatch lhs:', abs(m))
    print('CHECK test_builtin_magic_dispatch rhs:', 5)
    assert abs(m) == 5

    print('CHECK test_builtin_magic_dispatch lhs:', round(m))
    print('CHECK test_builtin_magic_dispatch rhs:', 5)
    assert round(m) == 5

    bs: bytes = bytes(m)
    print('CHECK test_builtin_magic_dispatch lhs:', len(bs))
    print('CHECK test_builtin_magic_dispatch rhs:', 5)
    assert len(bs) == 5

    it: MagicNum = iter(m)
    nxt: int = next(it)
    print('CHECK test_builtin_magic_dispatch lhs:', nxt)
    print('CHECK test_builtin_magic_dispatch rhs:', 5)
    assert nxt == 5


def test_set_str_and_empty_dict_set_calls() -> None:
    chars: list[str] = set("ab")
    d = dict()
    s = set()

    print('CHECK test_builtin_magic_dispatch lhs:', len(chars))
    print('CHECK test_builtin_magic_dispatch rhs:', 2)
    assert len(chars) == 2

    print('CHECK test_builtin_magic_dispatch lhs:', len(d))
    print('CHECK test_builtin_magic_dispatch rhs:', 0)
    assert len(d) == 0

    print('CHECK test_builtin_magic_dispatch lhs:', len(s))
    print('CHECK test_builtin_magic_dispatch rhs:', 0)
    assert len(s) == 0


def run_tests() -> None:
    test_builtin_magic_dispatch_paths()
    test_set_str_and_empty_dict_set_calls()
