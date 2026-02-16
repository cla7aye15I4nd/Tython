class CaptureBox:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def test_nested_direct_call() -> None:
    def add_one(x: int) -> int:
        return x + 1

    print('CHECK test_nested_function lhs:', add_one(41))
    print('CHECK test_nested_function rhs:', 42)
    assert add_one(41) == 42


def test_nested_with_multiple_params() -> None:
    def mix(a: int, b: int) -> int:
        return a * 10 + b

    print('CHECK test_nested_function lhs:', mix(3, 5))
    print('CHECK test_nested_function rhs:', 35)
    assert mix(3, 5) == 35


def test_nested_multi_level() -> None:
    def level1(x: int) -> int:
        def level2(y: int) -> int:
            def level3(z: int) -> int:
                return z + 5

            return level3(y * 2)

        return level2(x + 1)

    print('CHECK test_nested_function lhs:', level1(10))
    print('CHECK test_nested_function rhs:', 27)
    assert level1(10) == 27


def test_nested_in_function_scope() -> None:
    def apply_twice(v: int) -> int:
        def f(x: int) -> int:
            return x * 2

        return f(v) + f(1)

    print('CHECK test_nested_function lhs:', apply_twice(10))
    print('CHECK test_nested_function rhs:', 22)
    assert apply_twice(10) == 22


def test_nested_captures_value() -> None:
    base: int = 5

    def add_base(x: int) -> int:
        return x + base

    print('CHECK test_nested_function lhs:', add_base(7))
    print('CHECK test_nested_function rhs:', 12)
    assert add_base(7) == 12


def test_nested_deep_capture_chain() -> None:
    outer_bias: int = 3

    def level1(seed: int) -> int:
        l1_scale: int = 2

        def level2(step: int) -> int:
            l2_offset: int = 4

            def level3(mult: int) -> int:
                l3_shift: int = 1

                def level4(v: int) -> int:
                    return (v + l3_shift + l2_offset) * l1_scale + outer_bias

                return level4(step * mult)

            return level3(seed + step)

        return level2(seed - 1)

    print('CHECK test_nested_function lhs:', level1(5))
    print('CHECK test_nested_function rhs:', 85)
    assert level1(5) == 85


def test_nested_sibling_capture_from_parent() -> None:
    factor: int = 10

    def orchestrate(base: int) -> int:
        parent_adjust: int = 7

        def compute(x: int) -> int:
            return x * factor + parent_adjust

        def apply(y: int) -> int:
            return compute(y + 1) + parent_adjust

        return apply(base)

    print('CHECK test_nested_function lhs:', orchestrate(3))
    print('CHECK test_nested_function rhs:', 54)
    assert orchestrate(3) == 54


def test_nested_captures_class_instance() -> None:
    box: CaptureBox = CaptureBox(7)

    def read() -> int:
        return box.value

    print('CHECK test_nested_function lhs:', read())
    print('CHECK test_nested_function rhs:', 7)
    assert read() == 7


def run_tests() -> None:
    test_nested_direct_call()
    test_nested_with_multiple_params()
    test_nested_multi_level()
    test_nested_in_function_scope()
    test_nested_captures_value()
    test_nested_deep_capture_chain()
    test_nested_sibling_capture_from_parent()
    test_nested_captures_class_instance()
