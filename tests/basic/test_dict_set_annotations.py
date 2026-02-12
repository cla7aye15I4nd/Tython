def identity_dict(x: dict[str, list[int]]) -> dict[str, list[int]]:
    y: dict[str, list[int]] = x
    return y


def identity_set(x: set[tuple[int, int]]) -> set[tuple[int, int]]:
    y: set[tuple[int, int]] = x
    return y


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


def run_tests() -> None:
    test_dict_set_type_annotations_compile()
    test_nested_dict_set_annotations_compile()
