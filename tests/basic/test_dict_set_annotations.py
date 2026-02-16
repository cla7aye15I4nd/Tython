def identity_dict(x: dict[str, list[int]]) -> dict[str, list[int]]:
    y: dict[str, list[int]] = x
    return y


def identity_set(x: set[tuple[int, int]]) -> set[tuple[int, int]]:
    y: set[tuple[int, int]] = x
    return y


class Bucket:
    size: int

    def __init__(self, size: int) -> None:
        self.size = size


def test_dict_set_type_annotations_compile() -> None:
    ok: bool = True
    print('CHECK test_dict_set_annotations lhs:', ok)
    print('CHECK test_dict_set_annotations rhs:', True)
    assert ok


def test_nested_dict_set_annotations_compile() -> None:
    def outer_dict(x: dict[str, list[int]]) -> dict[str, list[int]]:
        def inner_dict(y: dict[str, list[int]]) -> dict[str, list[int]]:
            z: dict[str, list[int]] = y
            return z

        return inner_dict(x)

    def outer_set(x: set[tuple[int, int]]) -> set[tuple[int, int]]:
        def inner_set(y: set[tuple[int, int]]) -> set[tuple[int, int]]:
            z: set[tuple[int, int]] = y
            return z

        return inner_set(x)

    ok: bool = True
    print('CHECK test_dict_set_annotations lhs:', ok)
    print('CHECK test_dict_set_annotations rhs:', True)
    assert ok


def test_empty_literal_annotations() -> None:
    typed_list: list[int] = []
    typed_dict: dict[str, int] = {}
    typed_set: set[int] = set()

    print('CHECK test_dict_set_annotations lhs:', len(typed_list))
    print('CHECK test_dict_set_annotations rhs:', 0)
    assert len(typed_list) == 0
    print('CHECK test_dict_set_annotations lhs:', len(typed_dict))
    print('CHECK test_dict_set_annotations rhs:', 0)
    assert len(typed_dict) == 0
    print('CHECK test_dict_set_annotations lhs:', len(typed_set))
    print('CHECK test_dict_set_annotations rhs:', 0)
    assert len(typed_set) == 0


def test_tuple_from_generator_expression_annotation() -> None:
    generated: tuple[int, int] = tuple(i * 2 for i in [1, 2])

    print('CHECK test_dict_set_annotations lhs:', generated[0])
    print('CHECK test_dict_set_annotations rhs:', 2)
    assert generated[0] == 2
    print('CHECK test_dict_set_annotations lhs:', generated[1])
    print('CHECK test_dict_set_annotations rhs:', 4)
    assert generated[1] == 4


def test_assignments_to_subscript_and_attribute() -> None:
    values: list[int] = [10, 20]
    values[0] = 5
    bucket: Bucket = Bucket(1)
    bucket.size = 3

    print('CHECK test_dict_set_annotations lhs:', values[0])
    print('CHECK test_dict_set_annotations rhs:', 5)
    assert values[0] == 5
    print('CHECK test_dict_set_annotations lhs:', bucket.size)
    print('CHECK test_dict_set_annotations rhs:', 3)
    assert bucket.size == 3


def run_tests() -> None:
    test_dict_set_type_annotations_compile()
    test_nested_dict_set_annotations_compile()
    test_empty_literal_annotations()
    test_tuple_from_generator_expression_annotation()
    test_assignments_to_subscript_and_attribute()
