def test_list_int_literal() -> None:
    xs: list[int] = [1, 2, 3]
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 3)
    assert len(xs) == 3


def test_list_float_literal() -> None:
    xs: list[float] = [1.5, 2.5, 3.5]
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 3)
    assert len(xs) == 3


def test_list_bool_literal() -> None:
    xs: list[bool] = [True, False, True]
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 3)
    assert len(xs) == 3


def test_list_str_literal() -> None:
    xs: list[str] = ["hello", "world"]
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 2)
    assert len(xs) == 2


def test_list_empty() -> None:
    xs: list[int] = []
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 0)
    assert len(xs) == 0


def test_list_get() -> None:
    xs: list[int] = [10, 20, 30]
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 10)
    assert xs[0] == 10
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 20)
    assert xs[1] == 20
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 30)
    assert xs[2] == 30


def test_list_get_negative() -> None:
    xs: list[int] = [10, 20, 30]
    print('CHECK test_list lhs:', xs[-1])
    print('CHECK test_list rhs:', 30)
    assert xs[-1] == 30
    print('CHECK test_list lhs:', xs[-2])
    print('CHECK test_list rhs:', 20)
    assert xs[-2] == 20
    print('CHECK test_list lhs:', xs[-3])
    print('CHECK test_list rhs:', 10)
    assert xs[-3] == 10


def test_list_set() -> None:
    xs: list[int] = [1, 2, 3]
    xs[0] = 10
    xs[2] = 30
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 10)
    assert xs[0] == 10
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 2)
    assert xs[1] == 2
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 30)
    assert xs[2] == 30


def test_list_set_negative() -> None:
    xs: list[int] = [1, 2, 3]
    xs[-1] = 99
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 99)
    assert xs[2] == 99


def test_list_append_int() -> None:
    xs: list[int] = [1, 2]
    xs.append(3)
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 3)
    assert len(xs) == 3
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 3)
    assert xs[2] == 3


def test_list_append_float() -> None:
    xs: list[float] = [1.0, 2.0]
    xs.append(3.0)
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 3)
    assert len(xs) == 3
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 3.0)
    assert xs[2] == 3.0


def test_list_append_bool() -> None:
    xs: list[bool] = [True]
    xs.append(False)
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 2)
    assert len(xs) == 2
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', False)
    assert xs[1] == False


def test_list_clear() -> None:
    xs: list[int] = [1, 2, 3]
    xs.clear()
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 0)
    assert len(xs) == 0


def test_list_pop_int() -> None:
    xs: list[int] = [1, 2, 3]
    v: int = xs.pop()
    print('CHECK test_list lhs:', v)
    print('CHECK test_list rhs:', 3)
    assert v == 3
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 2)
    assert len(xs) == 2
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 2)
    assert xs[1] == 2


def test_list_pop_float() -> None:
    xs: list[float] = [1.5, 2.5]
    v: float = xs.pop()
    print('CHECK test_list lhs:', v)
    print('CHECK test_list rhs:', 2.5)
    assert v == 2.5
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 1)
    assert len(xs) == 1
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 1.5)
    assert xs[0] == 1.5


def test_list_pop_bool() -> None:
    xs: list[bool] = [True, False, True]
    v: bool = xs.pop()
    print('CHECK test_list lhs:', v)
    print('CHECK test_list rhs:', True)
    assert v == True
    print('CHECK test_list lhs:', len(xs))
    print('CHECK test_list rhs:', 2)
    assert len(xs) == 2
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', False)
    assert xs[1] == False


def test_list_truthiness() -> None:
    xs: list[int] = [1, 2, 3]
    if xs:
        print("truthy")
    ys: list[int] = []
    if ys:
        print("should not print")


def test_list_assert() -> None:
    xs: list[int] = [1]
    print('CHECK test_list assert expr:', 'xs')
    assert xs


def test_list_augmented_assign() -> None:
    xs: list[int] = [10, 20, 30]
    xs[0] += 5
    xs[1] -= 3
    xs[2] *= 2
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 15)
    assert xs[0] == 15
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 17)
    assert xs[1] == 17
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 60)
    assert xs[2] == 60


def test_list_float_get_set() -> None:
    xs: list[float] = [1.1, 2.2, 3.3]
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 1.1)
    assert xs[0] == 1.1
    xs[1] = 9.9
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 9.9)
    assert xs[1] == 9.9


def test_list_str_get() -> None:
    xs: list[str] = ["hello", "world"]
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 'hello')
    assert xs[0] == "hello"
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 'world')
    assert xs[1] == "world"


def test_list_print_int() -> None:
    print([1, 2, 3])


def test_list_print_float() -> None:
    print([1.5, 2.5])


def test_list_print_bool() -> None:
    print([True, False])


def test_list_copy_int() -> None:
    xs: list[int] = [3, 1, 2]
    ys: list[int] = xs.copy()
    ys[0] = 99
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', 3)
    assert xs[0] == 3
    print('CHECK test_list lhs:', ys[0])
    print('CHECK test_list rhs:', 99)
    assert ys[0] == 99


def test_list_extend_int() -> None:
    xs: list[int] = [1, 2]
    ys: list[int] = [3, 4]
    xs.extend(ys)
    print('CHECK test_list lhs:', xs)
    print('CHECK test_list rhs:', [1, 2, 3, 4])
    assert xs == [1, 2, 3, 4]
    print('CHECK test_list lhs:', ys)
    print('CHECK test_list rhs:', [3, 4])
    assert ys == [3, 4]


def test_list_sort_float() -> None:
    xs: list[float] = [3.5, -2.0, 1.25, 0.0]
    xs.sort()
    print('CHECK test_list lhs:', xs[0])
    print('CHECK test_list rhs:', -2.0)
    assert xs[0] == -2.0
    print('CHECK test_list lhs:', xs[1])
    print('CHECK test_list rhs:', 0.0)
    assert xs[1] == 0.0
    print('CHECK test_list lhs:', xs[2])
    print('CHECK test_list rhs:', 1.25)
    assert xs[2] == 1.25
    print('CHECK test_list lhs:', xs[3])
    print('CHECK test_list rhs:', 3.5)
    assert xs[3] == 3.5


def test_list_index_int() -> None:
    xs: list[int] = [4, 7, 9, 7]
    pos: int = xs.index(7)
    print('CHECK test_list lhs:', pos)
    print('CHECK test_list rhs:', 1)
    assert pos == 1


def test_list_count_int() -> None:
    xs: list[int] = [5, 5, 1, 5, 2]
    c: int = xs.count(5)
    print('CHECK test_list lhs:', c)
    print('CHECK test_list rhs:', 3)
    assert c == 3

def test_list_explicit_magic_and_methods() -> None:
    xs: list[int] = [3, 1, 2]
    xs.insert(1, 9)
    xs.remove(9)
    pos: int = xs.index(1)
    cnt: int = xs.count(3)
    xs.reverse()
    xs.sort()
    xs.extend([4, 5])

    copy_xs: list[int] = xs.copy()
    copy_xs[0] = 99
    print('CHECK test_list lhs:', xs)
    print('CHECK test_list rhs:', [1, 2, 3, 4, 5])
    assert xs == [1, 2, 3, 4, 5]
    print('CHECK test_list lhs:', pos)
    print('CHECK test_list rhs:', 1)
    assert pos == 1
    print('CHECK test_list lhs:', cnt)
    print('CHECK test_list rhs:', 1)
    assert cnt == 1
    print('CHECK test_list lhs:', copy_xs[0])
    print('CHECK test_list rhs:', 99)
    assert copy_xs[0] == 99

    plus: list[int] = xs.__add__([6])
    mul: list[int] = [7, 8].__mul__(2)
    inplace: list[int] = [1, 2]
    inplace_out: list[int] = inplace.__iadd__([3])
    imul_src: list[int] = [4, 5]
    imul_out: list[int] = imul_src.__imul__(2)
    del_src: list[int] = [10, 11, 12]
    del_src.__delitem__(1)
    contains_four: bool = xs.__contains__(4)
    third: int = xs.__getitem__(2)
    ln: int = xs.__len__()
    rev: list[int] = []
    for item in xs.__reversed__():
        rev.append(item)

    print('CHECK test_list lhs:', plus)
    print('CHECK test_list rhs:', [1, 2, 3, 4, 5, 6])
    assert plus == [1, 2, 3, 4, 5, 6]
    print('CHECK test_list lhs:', mul)
    print('CHECK test_list rhs:', [7, 8, 7, 8])
    assert mul == [7, 8, 7, 8]
    print('CHECK test_list lhs:', inplace_out)
    print('CHECK test_list rhs:', [1, 2, 3])
    assert inplace_out == [1, 2, 3]
    print('CHECK test_list lhs:', imul_out)
    print('CHECK test_list rhs:', [4, 5, 4, 5])
    assert imul_out == [4, 5, 4, 5]
    print('CHECK test_list lhs:', del_src)
    print('CHECK test_list rhs:', [10, 12])
    assert del_src == [10, 12]
    print('CHECK test_list lhs:', contains_four)
    print('CHECK test_list rhs:', True)
    assert contains_four == True
    print('CHECK test_list lhs:', third)
    print('CHECK test_list rhs:', 3)
    assert third == 3
    print('CHECK test_list lhs:', ln)
    print('CHECK test_list rhs:', 5)
    assert ln == 5
    print('CHECK test_list lhs:', rev)
    print('CHECK test_list rhs:', [5, 4, 3, 2, 1])
    assert rev == [5, 4, 3, 2, 1]


def test_list_more_magic_methods() -> None:
    xs: list[int] = [1, 2]

    same: bool = xs.__eq__([1, 2, 3])
    print('CHECK test_list lhs:', same)
    print('CHECK test_list rhs:', False)
    assert same == False


def test_list_binary_concat_and_repeat() -> None:
    left: list[int] = [1, 2]
    right: list[int] = [3, 4]
    concat: list[int] = left + right
    repeat1: list[int] = left * 3
    repeat2: list[int] = 2 * right

    print('CHECK test_list lhs:', concat)
    print('CHECK test_list rhs:', [1, 2, 3, 4])
    assert concat == [1, 2, 3, 4]
    print('CHECK test_list lhs:', repeat1)
    print('CHECK test_list rhs:', [1, 2, 1, 2, 1, 2])
    assert repeat1 == [1, 2, 1, 2, 1, 2]
    print('CHECK test_list lhs:', repeat2)
    print('CHECK test_list rhs:', [3, 4, 3, 4])
    assert repeat2 == [3, 4, 3, 4]


def test_list_slicing_paths() -> None:
    xs: list[int] = [0, 1, 2, 3, 4]
    s1: list[int] = xs[1:4]
    s2: list[int] = xs[:3]
    s3: list[int] = xs[2:]
    s4: list[int] = xs[:]
    s5: list[int] = xs[1:4:1]

    print('CHECK test_list lhs:', s1)
    print('CHECK test_list rhs:', [1, 2, 3])
    assert s1 == [1, 2, 3]
    print('CHECK test_list lhs:', s2)
    print('CHECK test_list rhs:', [0, 1, 2])
    assert s2 == [0, 1, 2]
    print('CHECK test_list lhs:', s3)
    print('CHECK test_list rhs:', [2, 3, 4])
    assert s3 == [2, 3, 4]
    print('CHECK test_list lhs:', s4)
    print('CHECK test_list rhs:', [0, 1, 2, 3, 4])
    assert s4 == [0, 1, 2, 3, 4]
    print('CHECK test_list lhs:', s5)
    print('CHECK test_list rhs:', [1, 2, 3])
    assert s5 == [1, 2, 3]


def run_tests() -> None:
    test_list_int_literal()
    test_list_float_literal()
    test_list_bool_literal()
    test_list_str_literal()
    test_list_empty()
    test_list_get()
    test_list_get_negative()
    test_list_set()
    test_list_set_negative()
    test_list_append_int()
    test_list_append_float()
    test_list_append_bool()
    test_list_pop_int()
    test_list_pop_float()
    test_list_pop_bool()
    test_list_clear()
    test_list_truthiness()
    test_list_assert()
    test_list_augmented_assign()
    test_list_float_get_set()
    test_list_str_get()
    test_list_print_int()
    test_list_print_float()
    test_list_print_bool()
    test_list_copy_int()
    test_list_extend_int()
    test_list_sort_float()
    test_list_index_int()
    test_list_count_int()
    test_list_explicit_magic_and_methods()
    test_list_more_magic_methods()
    test_list_binary_concat_and_repeat()
    test_list_slicing_paths()
