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

def run_tests() -> None:
    test_list_subscript_aug_add()
    test_list_subscript_aug_mul()
    test_list_subscript_aug_sub()
