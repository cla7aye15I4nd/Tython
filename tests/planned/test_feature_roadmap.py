"""
Roadmap feature specs.

This file defines expected behavior for features that are planned but not yet
fully implemented in Tython.
"""


def test_while_else_runs_only_without_break() -> None:
    i: int = 0
    out: list[int] = []
    while i < 3:
        out.append(i)
        i = i + 1
    else:
        out.append(99)
    print('CHECK test_feature_roadmap lhs:', out)
    print('CHECK test_feature_roadmap rhs:', [0, 1, 2, 99])
    assert out == [0, 1, 2, 99]

    j: int = 0
    out2: list[int] = []
    while j < 5:
        if j == 2:
            break
        out2.append(j)
        j = j + 1
    else:
        out2.append(99)
    print('CHECK test_feature_roadmap lhs:', out2)
    print('CHECK test_feature_roadmap rhs:', [0, 1])
    assert out2 == [0, 1]


def test_for_else_runs_only_without_break() -> None:
    out: list[int] = []
    for x in [1, 2, 3]:
        out.append(x)
    else:
        out.append(10)
    print('CHECK test_feature_roadmap lhs:', out)
    print('CHECK test_feature_roadmap rhs:', [1, 2, 3, 10])
    assert out == [1, 2, 3, 10]

    out2: list[int] = []
    for x in [1, 2, 3, 4]:
        if x == 3:
            break
        out2.append(x)
    else:
        out2.append(10)
    print('CHECK test_feature_roadmap lhs:', out2)
    print('CHECK test_feature_roadmap rhs:', [1, 2])
    assert out2 == [1, 2]


def test_try_else_runs_only_without_exception() -> None:
    marks: list[int] = []
    try:
        marks.append(1)
    except Exception:
        marks.append(2)
    else:
        marks.append(3)
    finally:
        marks.append(4)
    print('CHECK test_feature_roadmap lhs:', marks)
    print('CHECK test_feature_roadmap rhs:', [1, 3, 4])
    assert marks == [1, 3, 4]

    marks2: list[int] = []
    try:
        marks2.append(1)
        raise Exception("boom")
    except Exception:
        marks2.append(2)
    else:
        marks2.append(3)
    finally:
        marks2.append(4)
    print('CHECK test_feature_roadmap lhs:', marks2)
    print('CHECK test_feature_roadmap rhs:', [1, 2, 4])
    assert marks2 == [1, 2, 4]


def _reraiser() -> None:
    try:
        raise Exception("inner")
    except Exception:
        raise


def test_bare_raise_reraises_current_exception() -> None:
    seen: bool = False
    try:
        _reraiser()
    except Exception:
        seen = True
    print('CHECK test_feature_roadmap assert expr:', 'seen')
    assert seen


def test_exception_type_tags_beyond_base_and_stopiteration() -> None:
    saw_value_error: bool = False
    saw_runtime_error: bool = False

    try:
        raise ValueError("bad value")
    except ValueError:
        saw_value_error = True

    try:
        raise RuntimeError("runtime")
    except RuntimeError:
        saw_runtime_error = True

    print('CHECK test_feature_roadmap assert expr:', 'saw_value_error')
    assert saw_value_error
    print('CHECK test_feature_roadmap assert expr:', 'saw_runtime_error')
    assert saw_runtime_error


def test_list_methods_insert_remove_reverse_sort() -> None:
    xs: list[int] = [3, 1, 2, 1]
    xs.insert(1, 9)
    print('CHECK test_feature_roadmap lhs:', xs)
    print('CHECK test_feature_roadmap rhs:', [3, 9, 1, 2, 1])
    assert xs == [3, 9, 1, 2, 1]

    xs.remove(1)
    print('CHECK test_feature_roadmap lhs:', xs)
    print('CHECK test_feature_roadmap rhs:', [3, 9, 2, 1])
    assert xs == [3, 9, 2, 1]

    xs.reverse()
    print('CHECK test_feature_roadmap lhs:', xs)
    print('CHECK test_feature_roadmap rhs:', [1, 2, 9, 3])
    assert xs == [1, 2, 9, 3]

    xs.sort()
    print('CHECK test_feature_roadmap lhs:', xs)
    print('CHECK test_feature_roadmap rhs:', [1, 2, 3, 9])
    assert xs == [1, 2, 3, 9]


def test_bytearray_methods_insert_remove_reverse() -> None:
    ba: bytearray = bytearray(b"ace")
    ba.insert(1, 98)  # b
    print('CHECK test_feature_roadmap lhs:', ba)
    print('CHECK test_feature_roadmap rhs expr:', "bytearray(b'abce')")
    assert ba == bytearray(b"abce")

    ba.remove(99)  # c
    print('CHECK test_feature_roadmap lhs:', ba)
    print('CHECK test_feature_roadmap rhs expr:', "bytearray(b'abe')")
    assert ba == bytearray(b"abe")

    ba.reverse()
    print('CHECK test_feature_roadmap lhs:', ba)
    print('CHECK test_feature_roadmap rhs expr:', "bytearray(b'eba')")
    assert ba == bytearray(b"eba")


def test_builtins_sum_sorted_all_any() -> None:
    print('CHECK test_feature_roadmap lhs expr:', 'sum([1, 2, 3, 4])')
    print('CHECK test_feature_roadmap rhs:', 10)
    assert sum([1, 2, 3, 4]) == 10
    print('CHECK test_feature_roadmap lhs expr:', 'sum([1, 2, 3], 10)')
    print('CHECK test_feature_roadmap rhs:', 16)
    assert sum([1, 2, 3], 10) == 16

    print('CHECK test_feature_roadmap lhs expr:', 'sorted([3, 1, 2])')
    print('CHECK test_feature_roadmap rhs:', [1, 2, 3])
    assert sorted([3, 1, 2]) == [1, 2, 3]

    print('CHECK test_feature_roadmap lhs expr:', 'all([1, 1, 1])')
    print('CHECK test_feature_roadmap rhs:', True)
    assert all([1, 1, 1]) == True
    print('CHECK test_feature_roadmap lhs expr:', 'all([1, 0, 1])')
    print('CHECK test_feature_roadmap rhs:', False)
    assert all([1, 0, 1]) == False
    print('CHECK test_feature_roadmap lhs expr:', 'any([0, 0, 4])')
    print('CHECK test_feature_roadmap rhs:', True)
    assert any([0, 0, 4]) == True
    print('CHECK test_feature_roadmap lhs expr:', 'any([0, 0, 0])')
    print('CHECK test_feature_roadmap rhs:', False)
    assert any([0, 0, 0]) == False


def test_operator_extensions_in_and_is() -> None:
    xs: list[int] = [1, 2, 3]
    print('CHECK test_feature_roadmap lhs:', 2 in xs)
    print('CHECK test_feature_roadmap rhs:', True)
    assert (2 in xs) == True
    print('CHECK test_feature_roadmap lhs:', 9 in xs)
    print('CHECK test_feature_roadmap rhs:', False)
    assert (9 in xs) == False
    print('CHECK test_feature_roadmap lhs:', 9 not in xs)
    print('CHECK test_feature_roadmap rhs:', True)
    assert (9 not in xs) == True

    a: list[int] = xs
    b: list[int] = [1, 2, 3]
    print('CHECK test_feature_roadmap lhs:', a is xs)
    print('CHECK test_feature_roadmap rhs:', True)
    assert (a is xs) == True
    print('CHECK test_feature_roadmap lhs:', b is xs)
    print('CHECK test_feature_roadmap rhs:', False)
    assert (b is xs) == False
    print('CHECK test_feature_roadmap lhs:', b is not xs)
    print('CHECK test_feature_roadmap rhs:', True)
    assert (b is not xs) == True


def _complex_raise_on_target(n: int, target: int) -> int:
    if n == target:
        raise ValueError("target hit")
    return n * 2


def test_complex_nested_control_flow_roadmap() -> None:
    src: list[int] = [4, 3, 2, 1, 0]
    ordered: list[int] = sorted(src)
    print('CHECK test_feature_roadmap lhs:', ordered)
    print('CHECK test_feature_roadmap rhs:', [0, 1, 2, 3, 4])
    assert ordered == [0, 1, 2, 3, 4]

    transformed: list[int] = []
    marker: int = 0
    trace: list[int] = []

    for n in ordered:
        try:
            transformed.append(_complex_raise_on_target(n, 3))
        except ValueError:
            trace.append(100 + n)
            try:
                raise
            except ValueError:
                marker = marker + 10
        else:
            trace.append(200 + n)
        finally:
            trace.append(300 + n)
    else:
        marker = marker + 1

    print('CHECK test_feature_roadmap lhs:', transformed)
    print('CHECK test_feature_roadmap rhs:', [0, 2, 4, 8])
    assert transformed == [0, 2, 4, 8]
    print('CHECK test_feature_roadmap lhs:', trace)
    print('CHECK test_feature_roadmap rhs:', [200, 300, 201, 301, 202, 302, 103, 303, 204, 304])
    assert trace == [200, 300, 201, 301, 202, 302, 103, 303, 204, 304]
    print('CHECK test_feature_roadmap lhs:', marker)
    print('CHECK test_feature_roadmap rhs:', 11)
    assert marker == 11

    idx: int = 0
    seen_small: list[int] = []
    while idx < len(transformed):
        v: int = transformed[idx]
        if v in [0, 2, 4]:
            seen_small.append(v)
        idx = idx + 1
    else:
        marker = marker + 100

    print('CHECK test_feature_roadmap lhs:', seen_small)
    print('CHECK test_feature_roadmap rhs:', [0, 2, 4])
    assert seen_small == [0, 2, 4]
    print('CHECK test_feature_roadmap lhs:', marker)
    print('CHECK test_feature_roadmap rhs:', 111)
    assert marker == 111

    payload: bytearray = bytearray(b"ac")
    payload.insert(1, 98)  # b
    payload.append(100)  # d
    payload.remove(99)  # c
    payload.reverse()
    print('CHECK test_feature_roadmap lhs:', payload)
    print('CHECK test_feature_roadmap rhs expr:', "bytearray(b'dba')")
    assert payload == bytearray(b"dba")

    print('CHECK test_feature_roadmap lhs expr:', 'sum(seen_small)')
    print('CHECK test_feature_roadmap rhs:', 6)
    assert sum(seen_small) == 6
    print('CHECK test_feature_roadmap lhs expr:', 'all([marker == 111, len(payload) == 3, 4 in transformed])')
    print('CHECK test_feature_roadmap rhs:', True)
    assert all([marker == 111, len(payload) == 3, 4 in transformed]) == True
    print('CHECK test_feature_roadmap lhs expr:', 'any([9 in transformed, 8 in transformed, 5 in transformed])')
    print('CHECK test_feature_roadmap rhs:', True)
    assert any([9 in transformed, 8 in transformed, 5 in transformed]) == True


def run_tests() -> None:
    test_while_else_runs_only_without_break()
    test_for_else_runs_only_without_break()
    test_try_else_runs_only_without_exception()
    test_bare_raise_reraises_current_exception()
    test_exception_type_tags_beyond_base_and_stopiteration()
    test_list_methods_insert_remove_reverse_sort()
    test_bytearray_methods_insert_remove_reverse()
    test_builtins_sum_sorted_all_any()
    test_operator_extensions_in_and_is()
    test_complex_nested_control_flow_roadmap()
