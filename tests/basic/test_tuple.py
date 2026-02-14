def first_plus_len(t: tuple[int, int, int]) -> int:
    return t[0] + len(t)


class Box:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def add(self, delta: int) -> int:
        self.value = self.value + delta
        return self.value


def sum_by_rotating_index(t: tuple[int, int, int, int], steps: int) -> int:
    i: int = 0
    total: int = 0
    while i < steps:
        idx: int = (i * 7 + 3) % len(t)
        total = total + t[idx]
        i = i + 1
    return total


def walk_with_negative_indices(t: tuple[int, int, int, int, int]) -> int:
    i: int = -1
    total: int = 0
    while i >= -5:
        total = total + t[i]
        i = i - 1
    return total


def nested_tuple_list_class() -> int:
    nums: list[int] = [3, 5]
    box: Box = Box(10)
    nested: tuple[tuple[int, int], list[int], Box] = ((1, 2), nums, box)

    print('CHECK test_tuple lhs:', nested[0][0])
    print('CHECK test_tuple rhs:', 1)
    assert nested[0][0] == 1
    print('CHECK test_tuple lhs:', nested[0][1])
    print('CHECK test_tuple rhs:', 2)
    assert nested[0][1] == 2
    print('CHECK test_tuple lhs:', nested[1][0])
    print('CHECK test_tuple rhs:', 3)
    assert nested[1][0] == 3
    print('CHECK test_tuple lhs:', nested[2].value)
    print('CHECK test_tuple rhs:', 10)
    assert nested[2].value == 10

    nested[1].append(8)
    print('CHECK test_tuple lhs:', len(nested[1]))
    print('CHECK test_tuple rhs:', 3)
    assert len(nested[1]) == 3
    print('CHECK test_tuple lhs:', nested[1][2])
    print('CHECK test_tuple rhs:', 8)
    assert nested[1][2] == 8

    result: int = nested[2].add(nested[0][1])
    print('CHECK test_tuple lhs:', result)
    print('CHECK test_tuple rhs:', 12)
    assert result == 12
    print('CHECK test_tuple lhs:', nested[2].value)
    print('CHECK test_tuple rhs:', 12)
    assert nested[2].value == 12

    return nested[0][0] + nested[1][2] + nested[2].value


def test_tuple_static_index_with_dict_and_set() -> None:
    d: dict[int, int] = {1: 10, 2: 20}
    s: set[int] = {3, 4, 5}
    t: tuple[dict[int, int], set[int]] = (d, s)

    left_len: int = len(t[0])
    right_len: int = len(t[1])
    total: int = left_len + right_len

    print('CHECK test_tuple lhs:', left_len)
    print('CHECK test_tuple rhs:', 2)
    assert left_len == 2
    print('CHECK test_tuple lhs:', right_len)
    print('CHECK test_tuple rhs:', 3)
    assert right_len == 3
    print('CHECK test_tuple lhs:', total)
    print('CHECK test_tuple rhs:', 5)
    assert total == 5


def run_tests() -> None:
    nums: tuple[int, int, int] = (10, 20, 30)
    mixed: tuple[str, bytes, bool] = ("x", b"ab", True)
    idx: int = 1
    ints: tuple[int, int, int, int] = (4, 7, 11, 13)

    print('CHECK test_tuple lhs:', nums[1])
    print('CHECK test_tuple rhs:', 20)
    assert nums[1] == 20
    print('CHECK test_tuple lhs:', nums[-1])
    print('CHECK test_tuple rhs:', 30)
    assert nums[-1] == 30
    print('CHECK test_tuple lhs:', nums[idx])
    print('CHECK test_tuple rhs:', 20)
    assert nums[idx] == 20
    print('CHECK test_tuple lhs:', len(nums))
    print('CHECK test_tuple rhs:', 3)
    assert len(nums) == 3
    print('CHECK test_tuple lhs:', len(()))
    print('CHECK test_tuple rhs:', 0)
    assert len(()) == 0
    print('CHECK test_tuple lhs:', ints[idx + 1])
    print('CHECK test_tuple rhs:', 11)
    assert ints[(idx + 1)] == 11

    print('CHECK test_tuple lhs:', mixed[0])
    print('CHECK test_tuple rhs:', 'x')
    assert mixed[0] == "x"
    print('CHECK test_tuple lhs:', mixed[1])
    print('CHECK test_tuple rhs:', b'ab')
    assert mixed[1] == b"ab"
    print('CHECK test_tuple lhs:', mixed[2])
    print('CHECK test_tuple rhs:', True)
    assert mixed[2] == True
    res_fpl: int = first_plus_len(nums)
    print('CHECK test_tuple lhs:', res_fpl)
    print('CHECK test_tuple rhs:', 13)
    assert res_fpl == 13
    res_sbri: int = sum_by_rotating_index(ints, 25)
    print('CHECK test_tuple lhs:', res_sbri)
    print('CHECK test_tuple rhs:', 223)
    assert res_sbri == 223
    result1: int = walk_with_negative_indices((1, 2, 3, 4, 5))
    print('CHECK test_tuple lhs:', result1)
    print('CHECK test_tuple rhs:', 15)
    assert result1 == 15
    result2: int = nested_tuple_list_class()
    print('CHECK test_tuple lhs:', result2)
    print('CHECK test_tuple rhs:', 21)
    assert result2 == 21
    test_tuple_static_index_with_dict_and_set()

    if nums:
        print(1)
    else:
        print(0)

    if ():
        print(0)
    else:
        print(2)

    print(sum_by_rotating_index(ints, 12))
