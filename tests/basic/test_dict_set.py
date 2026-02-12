def test_dict_core_ops() -> None:
    d: dict[int, int] = {1: 10, 2: 20}
    print('CHECK test_dict_set lhs:', len(d))
    print('CHECK test_dict_set rhs:', 2)
    assert len(d) == 2

    print('CHECK test_dict_set lhs:', d[1])
    print('CHECK test_dict_set rhs:', 10)
    assert d[1] == 10

    d[1] = d[1] + 5
    print('CHECK test_dict_set lhs:', d[1])
    print('CHECK test_dict_set rhs:', 15)
    assert d[1] == 15

    d[2] += 3
    print('CHECK test_dict_set lhs:', d[2])
    print('CHECK test_dict_set rhs:', 23)
    assert d[2] == 23

    print('CHECK test_dict_set lhs:', 1 in d)
    print('CHECK test_dict_set rhs:', True)
    assert (1 in d) == True
    print('CHECK test_dict_set lhs:', 9 in d)
    print('CHECK test_dict_set rhs:', False)
    assert (9 in d) == False
    print('CHECK test_dict_set lhs:', 9 not in d)
    print('CHECK test_dict_set rhs:', True)
    assert (9 not in d) == True


def test_dict_methods_and_equality() -> None:
    a: dict[int, int] = {}
    a[3] = 30
    a[7] = 70

    print('CHECK test_dict_set lhs:', a.get(3))
    print('CHECK test_dict_set rhs:', 30)
    assert a.get(3) == 30

    p: int = a.pop(7)
    print('CHECK test_dict_set lhs:', p)
    print('CHECK test_dict_set rhs:', 70)
    assert p == 70

    b: dict[int, int] = a.copy()
    print('CHECK test_dict_set lhs:', a == b)
    print('CHECK test_dict_set rhs:', True)
    assert (a == b) == True

    b[3] = 31
    print('CHECK test_dict_set lhs:', a != b)
    print('CHECK test_dict_set rhs:', True)
    assert (a != b) == True

    a.clear()
    print('CHECK test_dict_set lhs:', len(a))
    print('CHECK test_dict_set rhs:', 0)
    assert len(a) == 0


def test_set_core_ops() -> None:
    s: set[int] = {1, 2, 3, 2}
    print('CHECK test_dict_set lhs:', len(s))
    print('CHECK test_dict_set rhs:', 3)
    assert len(s) == 3

    print('CHECK test_dict_set lhs:', 2 in s)
    print('CHECK test_dict_set rhs:', True)
    assert (2 in s) == True
    print('CHECK test_dict_set lhs:', 9 in s)
    print('CHECK test_dict_set rhs:', False)
    assert (9 in s) == False

    s.add(9)
    print('CHECK test_dict_set lhs:', 9 in s)
    print('CHECK test_dict_set rhs:', True)
    assert (9 in s) == True

    s.remove(2)
    print('CHECK test_dict_set lhs:', 2 in s)
    print('CHECK test_dict_set rhs:', False)
    assert (2 in s) == False

    s.discard(100)
    print('CHECK test_dict_set lhs:', len(s))
    print('CHECK test_dict_set rhs:', 3)
    assert len(s) == 3


def test_set_methods_and_equality() -> None:
    s0: set[int] = set()
    s0.add(5)
    s0.add(6)

    t: set[int] = s0.copy()
    print('CHECK test_dict_set lhs:', s0 == t)
    print('CHECK test_dict_set rhs:', True)
    assert (s0 == t) == True

    popped: int = t.pop()
    print('CHECK test_dict_set lhs:', popped == 5 or popped == 6)
    print('CHECK test_dict_set rhs:', True)
    assert (popped == 5 or popped == 6) == True

    print('CHECK test_dict_set lhs:', s0 != t)
    print('CHECK test_dict_set rhs:', True)
    assert (s0 != t) == True

    t.clear()
    print('CHECK test_dict_set lhs:', len(t))
    print('CHECK test_dict_set rhs:', 0)
    assert len(t) == 0


def run_tests() -> None:
    test_dict_core_ops()
    test_dict_methods_and_equality()
    test_set_core_ops()
    test_set_methods_and_equality()

