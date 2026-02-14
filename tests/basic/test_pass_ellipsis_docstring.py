"""Module-level docstring - should not generate code."""
...

def test_pass_basic() -> None:
    """Test basic pass statement."""
    pass
    print(1)

def test_pass_in_if() -> None:
    """Test pass in if/else blocks."""
    x: int = 5
    if x > 0:
        pass
    else:
        pass
    print(2)

def test_pass_in_loop() -> None:
    """Test pass in loops."""
    for i in range(3):
        pass
    print(3)

def test_ellipsis_basic() -> None:
    """Test basic ellipsis statement."""
    ...
    print(4)

def test_ellipsis_stub() -> None:
    """Test ellipsis in stub implementations."""
    if False:
        ...
    else:
        print(5)

def test_docstring_function() -> None:
    """This is a function docstring.

    It can be multi-line.
    """
    print(6)

def test_docstring_in_block() -> None:
    """Test docstrings in code blocks."""
    x: int = 10
    if x > 0:
        """This docstring should be ignored."""
        print(7)

def test_combined() -> None:
    """Test all three features together."""
    pass
    ...
    """Another docstring."""
    print(8)

def test_empty_function_with_pass() -> None:
    """Test function with only pass."""
    pass

def test_empty_function_with_ellipsis() -> None:
    """Test function with only ellipsis."""
    ...

def run_tests() -> None:
    test_pass_basic()
    test_pass_in_if()
    test_pass_in_loop()
    test_ellipsis_basic()
    test_ellipsis_stub()
    test_docstring_function()
    test_docstring_in_block()
    test_combined()
    test_empty_function_with_pass()
    test_empty_function_with_ellipsis()
    print("All tests passed!")

if __name__ == "__main__":
    run_tests()
