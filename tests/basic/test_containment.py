def test_in_list() -> None:
    xs: list[int] = [10, 20, 30, 40, 50]
    print('CHECK test_containment assert expr:', '30 in xs')
    assert 30 in xs
    print('CHECK test_containment assert expr:', 'not (99 in xs)')
    assert not (99 in xs)
    print("in_list ok")

def test_not_in_list() -> None:
    xs: list[int] = [1, 2, 3]
    print('CHECK test_containment assert expr:', '5 not in xs')
    assert 5 not in xs
    print('CHECK test_containment assert expr:', 'not (2 not in xs)')
    assert not (2 not in xs)
    print("not_in_list ok")

def test_in_str() -> None:
    s: str = "hello world"
    print('CHECK test_containment assert expr:', '"world" in s')
    assert "world" in s
    print('CHECK test_containment assert expr:', '"xyz" not in s')
    assert "xyz" not in s
    print('CHECK test_containment assert expr:', '"lo" in s')
    assert "lo" in s
    print("in_str ok")

def test_not_in_str() -> None:
    s: str = "abcdef"
    print('CHECK test_containment assert expr:', '"xyz" not in s')
    assert "xyz" not in s
    print('CHECK test_containment assert expr:', 'not ("cd" not in s)')
    assert not ("cd" not in s)
    print("not_in_str ok")

def run_tests() -> None:
    test_in_list()
    test_not_in_list()
    test_in_str()
    test_not_in_str()
