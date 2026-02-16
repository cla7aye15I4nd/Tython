class Plain:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def bump(x: int) -> int:
    return x + 1


def test_str_dict_single_item() -> None:
    payload: dict[int, int] = {5: 8}
    rendered: str = str(payload)
    print("CHECK test_intrinsic_str_edges lhs:", rendered)
    print("CHECK test_intrinsic_str_edges rhs:", "{5: 8}")
    assert rendered == "{5: 8}"


def test_str_set_single_item() -> None:
    payload: set[int] = {7}
    rendered: str = str(payload)
    print("CHECK test_intrinsic_str_edges lhs:", rendered)
    print("CHECK test_intrinsic_str_edges rhs:", "{7}")
    assert rendered == "{7}"


def test_str_nested_dict_and_set_in_list() -> None:
    nested_dicts: list[dict[int, int]] = [{5: 8}]
    nested_sets: list[set[int]] = [{7}]
    rendered_dicts: str = str(nested_dicts)
    rendered_sets: str = str(nested_sets)
    print("CHECK test_intrinsic_str_edges lhs:", rendered_dicts)
    print("CHECK test_intrinsic_str_edges rhs:", "[{5: 8}]")
    assert rendered_dicts == "[{5: 8}]"
    print("CHECK test_intrinsic_str_edges lhs:", rendered_sets)
    print("CHECK test_intrinsic_str_edges rhs:", "[{7}]")
    assert rendered_sets == "[{7}]"


def test_str_list_class_fallback_repr() -> None:
    payload: list[Plain] = [Plain(3)]
    rendered: str = str(payload)
    has_plain_object: bool = "Plain object" in rendered
    has_brackets: bool = "[<" in rendered and ">]" in rendered
    print("CHECK test_intrinsic_str_edges lhs:", has_plain_object)
    print("CHECK test_intrinsic_str_edges rhs:", True)
    assert has_plain_object
    print("CHECK test_intrinsic_str_edges lhs:", has_brackets)
    print("CHECK test_intrinsic_str_edges rhs:", True)
    assert has_brackets


def test_str_list_function_fallback_repr() -> None:
    payload = [bump]
    rendered: str = str(payload)
    has_shape: bool = "[<" in rendered and ">]" in rendered
    print("CHECK test_intrinsic_str_edges lhs:", has_shape)
    print("CHECK test_intrinsic_str_edges rhs:", True)
    assert has_shape


def test_str_list_function_variable_fallback_repr() -> None:
    f = bump
    payload = [f]
    rendered: str = str(payload)
    has_shape: bool = "[<" in rendered and ">]" in rendered
    print("CHECK test_intrinsic_str_edges lhs:", has_shape)
    print("CHECK test_intrinsic_str_edges rhs:", True)
    assert has_shape


def run_tests() -> None:
    test_str_dict_single_item()
    test_str_set_single_item()
    test_str_nested_dict_and_set_in_list()
    test_str_list_class_fallback_repr()
    test_str_list_function_fallback_repr()
    test_str_list_function_variable_fallback_repr()
