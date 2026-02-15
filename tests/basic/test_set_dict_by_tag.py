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

def test_dict_magic_methods_explicit_calls() -> None:
    d1: dict[int, int] = {1: 10, 2: 20}
    d2: dict[int, int] = {2: 20, 1: 10}
    d3: dict[int, int] = {1: 10, 2: 99}

    has_two: bool = d1.__contains__(2)
    has_three: bool = d1.__contains__(3)
    same: bool = d1.__eq__(d2)
    different: bool = d1.__ne__(d3)

    print("CHECK test_set_dict_by_tag lhs:", has_two)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert has_two == True
    print("CHECK test_set_dict_by_tag lhs:", has_three)
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert has_three == False
    print("CHECK test_set_dict_by_tag lhs:", same)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert same == True
    print("CHECK test_set_dict_by_tag lhs:", different)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert different == True


def test_set_magic_methods_explicit_calls() -> None:
    base: set[int] = {1, 2, 3}
    other: set[int] = {3, 4}

    print("CHECK test_set_dict_by_tag lhs:", base.__contains__(3))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__contains__(3) == True
    print("CHECK test_set_dict_by_tag lhs:", base.__eq__({1, 2, 3}))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__eq__({1, 2, 3}) == True
    print("CHECK test_set_dict_by_tag lhs:", base.__ne__(other))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__ne__(other) == True

    print("CHECK test_set_dict_by_tag lhs:", base.__and__(other) == {3})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__and__(other) == {3}
    print("CHECK test_set_dict_by_tag lhs:", base.__or__(other) == {1, 2, 3, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__or__(other) == {1, 2, 3, 4}
    print("CHECK test_set_dict_by_tag lhs:", base.__sub__(other) == {1, 2})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__sub__(other) == {1, 2}
    print("CHECK test_set_dict_by_tag lhs:", base.__xor__(other) == {1, 2, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__xor__(other) == {1, 2, 4}

    print("CHECK test_set_dict_by_tag lhs:", base.__rand__(other) == {3})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__rand__(other) == {3}
    print("CHECK test_set_dict_by_tag lhs:", base.__ror__(other) == {1, 2, 3, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__ror__(other) == {1, 2, 3, 4}
    print("CHECK test_set_dict_by_tag lhs:", base.__rsub__(other) == {4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__rsub__(other) == {4}
    print("CHECK test_set_dict_by_tag lhs:", base.__rxor__(other) == {1, 2, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert base.__rxor__(other) == {1, 2, 4}

    s1: set[int] = {1, 2, 3}
    s2: set[int] = {2, 4}
    s3: set[int] = {1, 2, 3}
    s4: set[int] = {2, 5}
    s5: set[int] = {1, 2, 3}
    s6: set[int] = {2, 3}
    s7: set[int] = {1, 2, 3}
    s8: set[int] = {2, 4}

    iand_res: set[int] = s1.__iand__(s2)
    print("CHECK test_set_dict_by_tag lhs:", iand_res == {2})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert iand_res == {2}
    ior_res: set[int] = s3.__ior__(s4)
    print("CHECK test_set_dict_by_tag lhs:", ior_res == {1, 2, 3, 5})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ior_res == {1, 2, 3, 5}
    isub_res: set[int] = s5.__isub__(s6)
    print("CHECK test_set_dict_by_tag lhs:", isub_res == {1})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert isub_res == {1}
    ixor_res: set[int] = s7.__ixor__(s8)
    print("CHECK test_set_dict_by_tag lhs:", ixor_res == {1, 3, 4})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert ixor_res == {1, 3, 4}

    copied_iter = base.__iter__()
    if copied_iter:
        pass
    print("CHECK test_set_dict_by_tag lhs:", base.__len__())
    print("CHECK test_set_dict_by_tag rhs:", 3)
    assert base.__len__() == 3


def test_dict_more_magic_methods() -> None:
    left: dict[int, int] = {1: 10}
    right: dict[int, int] = {2: 20}

    merged_left: dict[int, int] = left.__or__(right)
    merged_right: dict[int, int] = left.__ror__(right)
    print("CHECK test_set_dict_by_tag lhs:", merged_left == {1: 10, 2: 20})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert merged_left == {1: 10, 2: 20}
    print("CHECK test_set_dict_by_tag lhs:", merged_right == {2: 20, 1: 10})
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert merged_right == {2: 20, 1: 10}

    keys_seen: list[int] = []
    for k in left.__iter__():
        keys_seen.append(k)
    rev_seen: list[int] = []
    for k in left.__reversed__():
        rev_seen.append(k)
    print("CHECK test_set_dict_by_tag lhs:", keys_seen)
    print("CHECK test_set_dict_by_tag rhs:", [1])
    assert keys_seen == [1]
    print("CHECK test_set_dict_by_tag lhs:", rev_seen)
    print("CHECK test_set_dict_by_tag rhs:", [1])
    assert rev_seen == [1]


def test_dict_pop_default_and_magic_index_len() -> None:
    d: dict[int, int] = {1: 10}
    miss: int = d.pop(2, 99)
    print("CHECK test_set_dict_by_tag lhs:", miss)
    print("CHECK test_set_dict_by_tag rhs:", 99)
    assert miss == 99
    got: int = d.__getitem__(1)
    print("CHECK test_set_dict_by_tag lhs:", got)
    print("CHECK test_set_dict_by_tag rhs:", 10)
    assert got == 10
    ln: int = d.__len__()
    print("CHECK test_set_dict_by_tag lhs:", ln)
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert ln == 1


def test_set_identity_add_contains_discard_remove() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(1)
    s: set[NoEq] = set()
    s.add(a)
    s.add(b)
    s.add(a)
    print("CHECK test_set_dict_by_tag lhs:", len(s))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s) == 2
    print("CHECK test_set_dict_by_tag lhs:", a in s)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a in s) == True
    print("CHECK test_set_dict_by_tag lhs:", NoEq(1) in s)
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert (NoEq(1) in s) == False
    s.discard(a)
    print("CHECK test_set_dict_by_tag lhs:", len(s))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s) == 1
    s.add(a)
    s.remove(a)
    print("CHECK test_set_dict_by_tag lhs:", len(s))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s) == 1


def test_set_identity_algebra() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(2)
    c: NoEq = NoEq(3)
    s1: set[NoEq] = set()
    s1.add(a)
    s1.add(b)
    s2: set[NoEq] = set()
    s2.add(b)
    s2.add(c)
    print("CHECK test_set_dict_by_tag lhs:", len(s1.union(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 3)
    assert len(s1.union(s2)) == 3
    print("CHECK test_set_dict_by_tag lhs:", len(s1.intersection(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s1.intersection(s2)) == 1
    print("CHECK test_set_dict_by_tag lhs:", len(s1.difference(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s1.difference(s2)) == 1
    print("CHECK test_set_dict_by_tag lhs:", len(s1.symmetric_difference(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s1.symmetric_difference(s2)) == 2


def test_set_identity_update_ops() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(2)
    c: NoEq = NoEq(3)
    upd: set[NoEq] = set()
    upd.add(b)
    s1: set[NoEq] = set()
    s1.add(a)
    s1.update(upd)
    print("CHECK test_set_dict_by_tag lhs:", len(s1))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s1) == 2
    s2: set[NoEq] = set()
    s2.add(a)
    s2.add(b)
    s2.add(c)
    rem: set[NoEq] = set()
    rem.add(b)
    s2.difference_update(rem)
    print("CHECK test_set_dict_by_tag lhs:", len(s2))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s2) == 2
    s3: set[NoEq] = set()
    s3.add(a)
    s3.add(b)
    keep: set[NoEq] = set()
    keep.add(a)
    s3.intersection_update(keep)
    print("CHECK test_set_dict_by_tag lhs:", len(s3))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s3) == 1
    s4: set[NoEq] = set()
    s4.add(a)
    s4.add(b)
    other4: set[NoEq] = set()
    other4.add(b)
    other4.add(c)
    s4.symmetric_difference_update(other4)
    print("CHECK test_set_dict_by_tag lhs:", len(s4))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s4) == 2


def test_set_identity_relations() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(2)
    c: NoEq = NoEq(3)
    s1: set[NoEq] = set()
    s1.add(a)
    s2: set[NoEq] = set()
    s2.add(a)
    s2.add(b)
    s3: set[NoEq] = set()
    s3.add(c)
    s4: set[NoEq] = set()
    s4.add(a)
    print("CHECK test_set_dict_by_tag lhs:", s1.issubset(s2))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.issubset(s2) == True
    print("CHECK test_set_dict_by_tag lhs:", s2.issuperset(s1))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s2.issuperset(s1) == True
    print("CHECK test_set_dict_by_tag lhs:", s1.isdisjoint(s3))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.isdisjoint(s3) == True
    print("CHECK test_set_dict_by_tag lhs:", s1.isdisjoint(s2))
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert s1.isdisjoint(s2) == False
    print("CHECK test_set_dict_by_tag lhs:", s1.__eq__(s4))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.__eq__(s4) == True
    print("CHECK test_set_dict_by_tag lhs:", s1.__ne__(s2))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.__ne__(s2) == True
    print("CHECK test_set_dict_by_tag lhs:", s1.__lt__(s2))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.__lt__(s2) == True
    print("CHECK test_set_dict_by_tag lhs:", s1.__le__(s2))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.__le__(s2) == True
    print("CHECK test_set_dict_by_tag lhs:", s2.__gt__(s1))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s2.__gt__(s1) == True
    print("CHECK test_set_dict_by_tag lhs:", s2.__ge__(s1))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s2.__ge__(s1) == True
    print("CHECK test_set_dict_by_tag lhs:", s1.__contains__(a))
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert s1.__contains__(a) == True


def test_set_identity_magic_ops() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(2)
    c: NoEq = NoEq(3)
    s1: set[NoEq] = set()
    s1.add(a)
    s1.add(b)
    s2: set[NoEq] = set()
    s2.add(b)
    s2.add(c)
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__and__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s1.__and__(s2)) == 1
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__or__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 3)
    assert len(s1.__or__(s2)) == 3
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__sub__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s1.__sub__(s2)) == 1
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__xor__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s1.__xor__(s2)) == 2
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__rand__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s1.__rand__(s2)) == 1
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__ror__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 3)
    assert len(s1.__ror__(s2)) == 3
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__rsub__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(s1.__rsub__(s2)) == 1
    print("CHECK test_set_dict_by_tag lhs:", len(s1.__rxor__(s2)))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(s1.__rxor__(s2)) == 2


def test_set_identity_inplace_ops() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(2)
    c: NoEq = NoEq(3)
    ia: set[NoEq] = set()
    ia.add(a)
    ia.add(b)
    ia_o: set[NoEq] = set()
    ia_o.add(b)
    ia = ia.__iand__(ia_o)
    print("CHECK test_set_dict_by_tag lhs:", len(ia))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(ia) == 1
    io: set[NoEq] = set()
    io.add(a)
    io_o: set[NoEq] = set()
    io_o.add(c)
    io = io.__ior__(io_o)
    print("CHECK test_set_dict_by_tag lhs:", len(io))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(io) == 2
    isb: set[NoEq] = set()
    isb.add(a)
    isb.add(b)
    isb_o: set[NoEq] = set()
    isb_o.add(b)
    isb = isb.__isub__(isb_o)
    print("CHECK test_set_dict_by_tag lhs:", len(isb))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert len(isb) == 1
    ix: set[NoEq] = set()
    ix.add(a)
    ix.add(b)
    ix_o: set[NoEq] = set()
    ix_o.add(b)
    ix_o.add(c)
    ix = ix.__ixor__(ix_o)
    print("CHECK test_set_dict_by_tag lhs:", len(ix))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(ix) == 2


def test_dict_identity_key() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(1)
    d: dict[NoEq, int] = {}
    d[a] = 10
    d[b] = 20
    d[a] = 30
    print("CHECK test_set_dict_by_tag lhs:", len(d))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert len(d) == 2
    print("CHECK test_set_dict_by_tag lhs:", d[a])
    print("CHECK test_set_dict_by_tag rhs:", 30)
    assert d[a] == 30
    print("CHECK test_set_dict_by_tag lhs:", a in d)
    print("CHECK test_set_dict_by_tag rhs:", True)
    assert (a in d) == True
    print("CHECK test_set_dict_by_tag lhs:", NoEq(1) in d)
    print("CHECK test_set_dict_by_tag rhs:", False)
    assert (NoEq(1) in d) == False


def test_list_count_identity() -> None:
    a: NoEq = NoEq(1)
    b: NoEq = NoEq(1)
    xs: list[NoEq] = [a, b, a]
    print("CHECK test_set_dict_by_tag lhs:", xs.count(a))
    print("CHECK test_set_dict_by_tag rhs:", 2)
    assert xs.count(a) == 2
    print("CHECK test_set_dict_by_tag lhs:", xs.count(b))
    print("CHECK test_set_dict_by_tag rhs:", 1)
    assert xs.count(b) == 1


def run_tests() -> None:
    test_class_eq_identity_fallback()
    test_set_eq_by_tag_class()
    test_dict_key_and_value_eq_by_tag()
    test_dict_eq_by_tag()
    test_set_full_methods_int()
    test_dict_full_methods_int()
    test_dict_magic_methods_explicit_calls()
    test_set_magic_methods_explicit_calls()
    test_dict_more_magic_methods()
    test_dict_pop_default_and_magic_index_len()
    test_set_identity_add_contains_discard_remove()
    test_set_identity_algebra()
    test_set_identity_update_ops()
    test_set_identity_relations()
    test_set_identity_magic_ops()
    test_set_identity_inplace_ops()
    test_dict_identity_key()
    test_list_count_identity()
