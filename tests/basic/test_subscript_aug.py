def test_list_subscript_aug_add() -> None:
    xs: list[int] = [10, 20, 30]
    xs[0] += 5
    print('CHECK test_subscript_aug lhs:', xs[0])
    print('CHECK test_subscript_aug rhs:', 15)
    assert xs[0] == 15
    print("sub_aug_add ok")

def test_list_subscript_aug_mul() -> None:
    xs: list[int] = [10, 20, 30]
    xs[1] *= 3
    print('CHECK test_subscript_aug lhs:', xs[1])
    print('CHECK test_subscript_aug rhs:', 60)
    assert xs[1] == 60
    print("sub_aug_mul ok")

def test_list_subscript_aug_sub() -> None:
    xs: list[int] = [100, 200, 300]
    xs[2] -= 50
    print('CHECK test_subscript_aug lhs:', xs[2])
    print('CHECK test_subscript_aug rhs:', 250)
    assert xs[2] == 250
    print("sub_aug_sub ok")


def test_dict_subscript_aug_add() -> None:
    d: dict[int, int] = {1: 10, 2: 20}
    d[1] += 7
    print('CHECK test_subscript_aug lhs:', d[1])
    print('CHECK test_subscript_aug rhs:', 17)
    assert d[1] == 17
    print("dict_sub_aug_add ok")


def run_tests() -> None:
    test_list_subscript_aug_add()
    test_list_subscript_aug_mul()
    test_list_subscript_aug_sub()
    test_dict_subscript_aug_add()
