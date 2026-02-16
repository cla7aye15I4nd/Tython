def test_set_float_contains() -> None:
    s: set[float] = {1.5, 2.5}
    has_a: bool = 2.5 in s
    has_b: bool = 3.5 in s
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", has_a)
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", True)
    assert has_a
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", has_b)
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", False)
    assert not has_b


def test_dict_float_lookup_update() -> None:
    d: dict[float, int] = {1.5: 10}
    d[2.5] = 20
    d[1.5] = 11
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", d[1.5])
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", 11)
    assert d[1.5] == 11
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", d[2.5])
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", 20)
    assert d[2.5] == 20


def test_list_float_equality() -> None:
    a: list[float] = [1.5, 2.5]
    b: list[float] = [1.5, 2.5]
    c: list[float] = [1.5, 3.5]
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", a == b)
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", True)
    assert a == b
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", a == c)
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", False)
    assert not (a == c)


def test_list_set_eq_safe() -> None:
    x: set[int] = {1}
    y: set[int] = {2}
    a: list[set[int]] = [x]
    b: list[set[int]] = [x]
    c: list[set[int]] = [y]
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", a == b)
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", True)
    assert a == b
    print("CHECK test_intrinsic_cmp_hash_edges lhs:", a == c)
    print("CHECK test_intrinsic_cmp_hash_edges rhs:", False)
    assert not (a == c)


def run_tests() -> None:
    test_set_float_contains()
    test_dict_float_lookup_update()
    test_list_float_equality()
    test_list_set_eq_safe()
