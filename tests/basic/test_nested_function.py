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


def run_tests() -> None:
    test_nested_direct_call()
    test_nested_with_multiple_params()
    test_nested_multi_level()
