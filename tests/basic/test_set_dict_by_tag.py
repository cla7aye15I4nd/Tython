class EqKey:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __eq__(self, other: "EqKey") -> bool:
        return self.value == other.value

    def __hash__(self) -> int:
        return self.value


class EqVal:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __eq__(self, other: "EqVal") -> bool:
        return self.value == other.value


class NoEq:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def test_class_eq_identity_fallback() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(1)
    c: NoEq = a
    print("CHECK test_set_dict_by_tag lhs:", a == c)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a == c) == True
    print("CHECK test_set_dict_by_tag lhs:", a == b)
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert (a == b) == False
    xs: list[NoEq] = [a]
    print("CHECK test_set_dict_by_tag lhs:", a in xs)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a in xs) == True
    print("CHECK test_set_dict_by_tag lhs:", NoEq(1) in xs)
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert (NoEq(1) in xs) == False


def test_set_eq_by_tag_class() -> None:
    s: set[EqKey] = set()
    s.add(EqKey(1))
    s.add(EqKey(1))
    print("CHECK test_set_dict_by_tag lhs:", len(s))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s) == 1
    print("CHECK test_set_dict_by_tag lhs:", EqKey(1) in s)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (EqKey(1) in s) == True
    s.discard(EqKey(2))
    print("CHECK test_set_dict_by_tag lhs:", len(s))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s) == 1
    s.remove(EqKey(1))
    print("CHECK test_set_dict_by_tag lhs:", len(s))
    print("CHECK test_set_dict_by_tag rhs:", 0)
    assert len(s) == 0

    a: set[EqKey] = {EqKey(2), EqKey(3)}
    b: set[EqKey] = {EqKey(3), EqKey(2)}
    print("CHECK test_set_dict_by_tag lhs:", a == b)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a == b) == True


def test_dict_key_and_value_eq_by_tag() -> None:
    d: dict[EqKey, EqVal] = {}
    d[EqKey(1)] = EqVal(10)
    d[EqKey(1)] = EqVal(20)
    print("CHECK test_set_dict_by_tag lhs:", len(d))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(d) == 1
    print("CHECK test_set_dict_by_tag lhs:", EqKey(1) in d)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (EqKey(1) in d) == True
    got: EqVal = d[EqKey(1)]
    print("CHECK test_set_dict_by_tag lhs:", got.value)
    print("CHECK test_set_dict_by_tag rhs:", 20)
    assert got.value == 20
    popped: EqVal = d.pop(EqKey(1))
    print("CHECK test_set_dict_by_tag lhs:", popped.value)
    print("CHECK test_set_dict_by_tag rhs:", 20)
    assert popped.value == 20
    print("CHECK test_set_dict_by_tag lhs:", len(d))
    print("CHECK test_set_dict_by_tag rhs:", 0)
    assert len(d) == 0


def test_dict_eq_by_tag() -> None:
    a: dict[EqKey, EqVal] = {EqKey(1): EqVal(7), EqKey(2): EqVal(9)}
    b: dict[EqKey, EqVal] = {EqKey(2): EqVal(9), EqKey(1): EqVal(7)}
    c: dict[EqKey, EqVal] = {EqKey(1): EqVal(7), EqKey(2): EqVal(8)}
    print("CHECK test_set_dict_by_tag lhs:", a == b)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a == b) == True
    print("CHECK test_set_dict_by_tag lhs:", a == c)
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert (a == c) == False


def test_set_full_methods_int() -> None:
    a: set[int] = {1, 2, 3}
    b: set[int] = {3, 4}
    print("CHECK test_set_dict_by_tag lhs:", a.union(b) == {1, 2, 3, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a.union(b) == {1, 2, 3, 4}) == True
    print("CHECK test_set_dict_by_tag lhs:", a.intersection(b) == {3})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a.intersection(b) == {3}) == True
    print("CHECK test_set_dict_by_tag lhs:", a.difference(b) == {1, 2})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a.difference(b) == {1, 2}) == True
    print("CHECK test_set_dict_by_tag lhs:", a.symmetric_difference(b) == {1, 2, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a.symmetric_difference(b) == {1, 2, 4}) == True
    print("CHECK test_set_dict_by_tag lhs:", a.isdisjoint({7, 8}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a.isdisjoint({7, 8})) == True
    print("CHECK test_set_dict_by_tag lhs:", {1, 2}.issubset(a))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ({1, 2}.issubset(a)) == True
    print("CHECK test_set_dict_by_tag lhs:", a.issuperset({1, 2}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a.issuperset({1, 2})) == True

    c: set[int] = {1, 2, 3}
    c.update({4, 5})
    c.intersection_update({2, 3, 5, 9})
    c.difference_update({9})
    c.symmetric_difference_update({3, 7})
    print("CHECK test_set_dict_by_tag lhs:", c == {2, 5, 7})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (c == {2, 5, 7}) == True

    print("CHECK test_set_dict_by_tag lhs:", ({1, 2}).__lt__({1, 2, 3}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ({1, 2}).__lt__({1, 2, 3}) == True
    print("CHECK test_set_dict_by_tag lhs:", ({1, 2}).__le__({1, 2}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ({1, 2}).__le__({1, 2}) == True
    print("CHECK test_set_dict_by_tag lhs:", ({1, 2, 3}).__gt__({1, 2}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ({1, 2, 3}).__gt__({1, 2}) == True
    print("CHECK test_set_dict_by_tag lhs:", ({1, 2, 3}).__ge__({1, 2, 3}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ({1, 2, 3}).__ge__({1, 2, 3}) == True


def test_dict_full_methods_int() -> None:
    d: dict[int, int] = {1: 10, 2: 20}
    print("CHECK test_set_dict_by_tag lhs:", d.get(9, 99))
    print("CHECK test_set_dict_by_tag rhs:", 99)
    assert d.get(9, 99) == 99
    print("CHECK test_set_dict_by_tag lhs:", sorted(d.keys()))
    print("CHECK test_set_dict_by_tag rhs:", [1, 2])
    assert sorted(d.keys()) == [1, 2]
    print("CHECK test_set_dict_by_tag lhs:", sorted(d.values()))
    print("CHECK test_set_dict_by_tag rhs:", [10, 20])
    assert sorted(d.values()) == [10, 20]
    items: list[tuple[int, int]] = d.items()
    print("CHECK test_set_dict_by_tag lhs:", len(items))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(items) == 2
    print("CHECK test_set_dict_by_tag lhs:", (1, 10) in items)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ((1, 10) in items) == True
    print("CHECK test_set_dict_by_tag lhs:", (2, 20) in items)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ((2, 20) in items) == True

    s: dict[int, int] = d.fromkeys([7, 8], 3)
    print("CHECK test_set_dict_by_tag lhs:", s == {7: 3, 8: 3})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (s == {7: 3, 8: 3}) == True

    d.setdefault(3, 30)
    d.setdefault(3, 300)
    print("CHECK test_set_dict_by_tag lhs:", d[3])
    print("CHECK test_set_dict_by_tag rhs:", 30)
    assert d[3] == 30

    d.update({4: 40})
    print("CHECK test_set_dict_by_tag lhs:", d[4])
    print("CHECK test_set_dict_by_tag rhs:", 40)
    assert d[4] == 40

    e: dict[int, int] = d.__or__({5: 50})
    print("CHECK test_set_dict_by_tag lhs:", e[5])
    print("CHECK test_set_dict_by_tag rhs:", 50)
    assert e[5] == 50
    d = d.__ior__({6: 60})
    print("CHECK test_set_dict_by_tag lhs:", d[6])
    print("CHECK test_set_dict_by_tag rhs:", 60)
    assert d[6] == 60

    x: dict[int, int] = {1: 1}
    x.__setitem__(2, 2)
    x.__delitem__(1)
    print("CHECK test_set_dict_by_tag lhs:", x == {2: 2})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (x == {2: 2}) == True

    p: dict[int, int] = {9: 90, 10: 100}
    pair: tuple[int, int] = p.popitem()
    print("CHECK test_set_dict_by_tag lhs:", pair)
    print("CHECK test_set_dict_by_tag rhs:", (10, 100))
    assert pair == (10, 100)


def run_tests() -> None:
    test_class_eq_identity_fallback()
    test_set_eq_by_tag_class()
    test_dict_key_and_value_eq_by_tag()
    test_dict_eq_by_tag()
    test_set_full_methods_int()
    test_dict_full_methods_int()
