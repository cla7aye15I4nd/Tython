def test_nested_direct_call() -> None:
    def add_one(x: int) -> int:
        return x + 1

    assert add_one(41) == 42


def test_nested_callable_roundtrip() -> None:
    def mul2(x: int) -> int:
        return x * 2

    fn: "callable[[int], int]" = mul2
    assert fn(7) == 14


def test_nested_with_multiple_params() -> None:
    def mix(a: int, b: int) -> int:
        return a * 10 + b

    assert mix(3, 5) == 35


def test_nested_multi_level() -> None:
    def level1(x: int) -> int:
        def level2(y: int) -> int:
            def level3(z: int) -> int:
                return z + 5

            return level3(y * 2)

        return level2(x + 1)

    assert level1(10) == 27


def test_nested_callable_as_argument() -> None:
    def apply_twice(f: "callable[[int], int]", x: int) -> int:
        return f(f(x))

    def inc(v: int) -> int:
        return v + 1

    assert apply_twice(inc, 5) == 7


def test_nested_callable_as_return_type() -> None:
    def make_worker(flag: int) -> "callable[[int], int]":
        def plus_two(x: int) -> int:
            return x + 2

        def times_three(x: int) -> int:
            return x * 3

        if flag == 0:
            return plus_two
        return times_three

    first: "callable[[int], int]" = make_worker(0)
    second: "callable[[int], int]" = make_worker(1)
    assert first(4) == 6
    assert second(4) == 12


def run_tests() -> None:
    test_nested_direct_call()
    test_nested_callable_roundtrip()
    test_nested_with_multiple_params()
    test_nested_multi_level()
    test_nested_callable_as_argument()
    test_nested_callable_as_return_type()
