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

    assert nested[0][0] == 1
    assert nested[0][1] == 2
    assert nested[1][0] == 3
    assert nested[2].value == 10

    nested[1].append(8)
    assert len(nested[1]) == 3
    assert nested[1][2] == 8

    assert nested[2].add(nested[0][1]) == 12
    assert nested[2].value == 12

    return nested[0][0] + nested[1][2] + nested[2].value


def run_tests() -> None:
    nums: tuple[int, int, int] = (10, 20, 30)
    mixed: tuple[str, bytes, bool] = ("x", b"ab", True)
    idx: int = 1
    ints: tuple[int, int, int, int] = (4, 7, 11, 13)

    assert nums[1] == 20
    assert nums[-1] == 30
    assert nums[idx] == 20
    assert len(nums) == 3
    assert len(()) == 0
    assert ints[(idx + 1)] == 11

    assert mixed[0] == "x"
    assert mixed[1] == b"ab"
    assert mixed[2] == True
    assert first_plus_len(nums) == 13
    assert sum_by_rotating_index(ints, 25) == 223
    assert walk_with_negative_indices((1, 2, 3, 4, 5)) == 15
    assert nested_tuple_list_class() == 21

    if nums:
        print(1)
    else:
        print(0)

    if ():
        print(0)
    else:
        print(2)

    print(sum_by_rotating_index(ints, 12))
