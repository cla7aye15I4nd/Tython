def add3(a: int, b: int = 10, c: int = 100) -> int:
    return a + b + c


def test_defaults_and_keywords_top_level() -> None:
    print('CHECK test_function_call_args lhs:', add3(1))
    print('CHECK test_function_call_args rhs:', 111)
    assert add3(1) == 111

    print('CHECK test_function_call_args lhs:', add3(1, c=5))
    print('CHECK test_function_call_args rhs:', 16)
    assert add3(1, c=5) == 16

    print('CHECK test_function_call_args lhs:', add3(a=2, b=3, c=4))
    print('CHECK test_function_call_args rhs:', 9)
    assert add3(a=2, b=3, c=4) == 9


def test_defaults_and_keywords_nested() -> None:
    def mix(a: int, b: int = 7) -> int:
        return a * 10 + b

    print('CHECK test_function_call_args lhs:', mix(3))
    print('CHECK test_function_call_args rhs:', 37)
    assert mix(3) == 37

    print('CHECK test_function_call_args lhs:', mix(a=4, b=2))
    print('CHECK test_function_call_args rhs:', 42)
    assert mix(a=4, b=2) == 42


def test_nested_keyword_reorder() -> None:
    def compute(a: int, b: int = 5, c: int = 9) -> int:
        return a + b * c

    print('CHECK test_function_call_args lhs:', compute(c=2, a=4, b=3))
    print('CHECK test_function_call_args rhs:', 10)
    assert compute(c=2, a=4, b=3) == 10


def test_nested_multi_level_with_defaults() -> None:
    def level1(a: int, b: int = 1) -> int:
        def level2(c: int, d: int = 2) -> int:
            def level3(e: int, f: int = 3) -> int:
                return e + f

            return level3(e=c, f=d)

        return level2(c=a, d=b)

    print('CHECK test_function_call_args lhs:', level1(7))
    print('CHECK test_function_call_args rhs:', 8)
    assert level1(7) == 8

    print('CHECK test_function_call_args lhs:', level1(a=6, b=4))
    print('CHECK test_function_call_args rhs:', 10)
    assert level1(a=6, b=4) == 10


def test_primitive_arg_coercions() -> None:
    def takes_float(x: float) -> float:
        return x

    def takes_int(x: int) -> int:
        return x

    def takes_bool(x: bool) -> bool:
        return bool(x)

    print('CHECK test_function_call_args lhs:', takes_float(3) == 3.0)
    print('CHECK test_function_call_args rhs:', True)
    assert takes_float(3) == 3.0
    print('CHECK test_function_call_args lhs:', takes_float(True) == 1.0)
    print('CHECK test_function_call_args rhs:', True)
    assert takes_float(True) == 1.0

    print('CHECK test_function_call_args lhs:', takes_int(False) == 0)
    print('CHECK test_function_call_args rhs:', True)
    assert takes_int(False) == 0

    print('CHECK test_function_call_args lhs:', takes_bool(0.0) == False)
    print('CHECK test_function_call_args rhs:', True)
    assert takes_bool(0.0) == False
    print('CHECK test_function_call_args lhs:', takes_bool(7) == True)
    print('CHECK test_function_call_args rhs:', True)
    assert takes_bool(7) == True


def test_default_expression_matrix() -> None:
    def defaults(
        a: int = -1,
        b: float = -1.5,
        c: bool = 1 == 1,
        d: int = ~1,
        e: int = 1 + 2 * 3,
        f: int = 9 // 2,
        g: int = 9 % 4,
        h: int = 2 << 3,
        i: bool = 3 > 2 and 4 >= 4,
        j: tuple[int, int] = (1, 2),
        k: list[int] = [3, 4],
    ) -> int:
        if c and i:
            return a + d + e + f + g + h + j[0] + j[1] + k[0] + k[1]
        return 0

    print('CHECK test_function_call_args lhs:', defaults())
    print('CHECK test_function_call_args rhs:', 35)
    assert defaults() == 35
    print('CHECK test_function_call_args lhs:', defaults(10, -1.5, True, ~0, 4, 3, 2, 1, True, (5, 6), [7, 8]))
    print('CHECK test_function_call_args rhs:', 45)
    assert defaults(10, -1.5, True, ~0, 4, 3, 2, 1, True, (5, 6), [7, 8]) == 45


def test_empty_list_default_param() -> None:
    def size(xs: list[int] = []) -> int:
        return len(xs)

    print('CHECK test_function_call_args lhs:', size())
    print('CHECK test_function_call_args rhs:', 0)
    assert size() == 0
    print('CHECK test_function_call_args lhs:', size([1, 2, 3]))
    print('CHECK test_function_call_args rhs:', 3)
    assert size([1, 2, 3]) == 3


def test_default_expression_float_cmp_matrix() -> None:
    def probe(
        fa: float = 1.0 + 2.0,
        fb: float = 5.0 - 1.5,
        fc: float = 2.0 * 3.0,
        fd: float = 7.5 / 2.5,
        fe: float = 7.5 // 2.0,
        ff: float = 7.5 % 2.0,
        fg: float = 2.0 ** 3.0,
        b0: bool = 1 == 1,
        b1: bool = 1 != 2,
        b2: bool = 1 < 2,
        b3: bool = 1 <= 2,
        b4: bool = 2 > 1,
        b5: bool = 2 >= 1,
        c0: bool = 1.0 == 1.0,
        c1: bool = 1.0 != 2.0,
        c2: bool = 1.0 < 2.0,
        c3: bool = 1.0 <= 2.0,
        c4: bool = 2.0 > 1.0,
        c5: bool = 2.0 >= 1.0,
    ) -> int:
        if b0 and b1 and b2 and b3 and b4 and b5 and c0 and c1 and c2 and c3 and c4 and c5:
            if fa > 0.0 and fb > 0.0 and fc > 0.0 and fd > 0.0 and fe > 0.0 and ff > 0.0 and fg > 0.0:
                return 1
        return 0

    print('CHECK test_function_call_args lhs:', probe())
    print('CHECK test_function_call_args rhs:', 1)
    assert probe() == 1


def test_default_expression_bitwise_bool_and_cast() -> None:
    def probe(
        a: int = 9 - 4,
        b: int = 2 ** 5,
        c: int = 6 & 3,
        d: int = 6 | 1,
        e: int = 6 ^ 3,
        f: int = 8 >> 2,
        g: bool = 1 == 1,
        h: bool = 1 != 2,
        i: bool = (1 < 2) and (3 > 2),
        j: bool = (1 == 2) or (2 == 2),
        k: float = 1 + 2.0,
        m: float = -1.5,
        n: bool = not (1 == 2),
        o: bool = (1 == 1) == (2 == 2),
        p: bool = (1 == 1) != (2 == 3),
    ) -> int:
        if g and h and i and j and n and o and p and k > 0.0 and m < 0.0:
            return a + b + c + d + e + f
        return 0

    print('CHECK test_function_call_args lhs:', probe())
    print('CHECK test_function_call_args rhs:', 53)
    assert probe() == 53


def run_tests() -> None:
    test_defaults_and_keywords_top_level()
    test_defaults_and_keywords_nested()
    test_nested_keyword_reorder()
    test_nested_multi_level_with_defaults()
    test_primitive_arg_coercions()
    test_default_expression_matrix()
    test_empty_list_default_param()
    test_default_expression_float_cmp_matrix()
    test_default_expression_bitwise_bool_and_cast()
