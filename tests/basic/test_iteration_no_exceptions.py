"""
Test that str, bytes, and bytearray iteration works WITHOUT exception handling.
This verifies that iteration is implemented using direct indexing instead of
the __iter__/__next__ protocol with StopIteration.
"""

def test_str_iteration_basic() -> None:
    """Test basic string iteration"""
    s: str = "hello"
    result: list[str] = []

    for ch in s:
        result.append(ch)

    assert len(result) == 5
    assert result[0] == "h"
    assert result[1] == "e"
    assert result[2] == "l"
    assert result[3] == "l"
    assert result[4] == "o"
    print("✓ test_str_iteration_basic passed")

def test_str_iteration_empty() -> None:
    """Test iteration over empty string"""
    s: str = ""
    count: int = 0

    for ch in s:
        count += 1

    assert count == 0
    print("✓ test_str_iteration_empty passed")

def test_str_iteration_single() -> None:
    """Test iteration over single character string"""
    s: str = "a"
    result: str = ""

    for ch in s:
        result = ch

    assert result == "a"
    print("✓ test_str_iteration_single passed")

def test_bytes_iteration_basic() -> None:
    """Test basic bytes iteration - returns int values"""
    b: bytes = b"xyz"
    result: list[int] = []

    for byte_val in b:
        result.append(byte_val)

    assert len(result) == 3
    assert result[0] == 120  # ord('x')
    assert result[1] == 121  # ord('y')
    assert result[2] == 122  # ord('z')
    print("✓ test_bytes_iteration_basic passed")

def test_bytes_iteration_empty() -> None:
    """Test iteration over empty bytes"""
    b: bytes = b""
    count: int = 0

    for byte_val in b:
        count += 1

    assert count == 0
    print("✓ test_bytes_iteration_empty passed")

def test_bytes_iteration_values() -> None:
    """Test that bytes iteration returns correct integer values"""
    b: bytes = b"\x00\x01\xff"
    result: list[int] = []

    for byte_val in b:
        result.append(byte_val)

    assert result[0] == 0
    assert result[1] == 1
    assert result[2] == 255
    print("✓ test_bytes_iteration_values passed")

def test_bytearray_iteration_basic() -> None:
    """Test basic bytearray iteration - returns int values"""
    ba: bytearray = bytearray(b"abc")
    result: list[int] = []

    for byte_val in ba:
        result.append(byte_val)

    assert len(result) == 3
    assert result[0] == 97   # ord('a')
    assert result[1] == 98   # ord('b')
    assert result[2] == 99   # ord('c')
    print("✓ test_bytearray_iteration_basic passed")

def test_bytearray_iteration_empty() -> None:
    """Test iteration over empty bytearray"""
    ba: bytearray = bytearray(b"")
    count: int = 0

    for byte_val in ba:
        count += 1

    assert count == 0
    print("✓ test_bytearray_iteration_empty passed")

def test_bytearray_iteration_modified() -> None:
    """Test bytearray iteration after modification"""
    ba: bytearray = bytearray(b"test")

    # Modify the bytearray
    ba.append(33)  # Add '!'

    result: list[int] = []
    for byte_val in ba:
        result.append(byte_val)

    assert len(result) == 5
    assert result[4] == 33
    print("✓ test_bytearray_iteration_modified passed")

def test_str_iteration_with_break() -> None:
    """Test string iteration with break statement"""
    s: str = "abcdef"
    result: list[str] = []

    for ch in s:
        if ch == "d":
            break
        result.append(ch)

    assert len(result) == 3
    assert result[0] == "a"
    assert result[1] == "b"
    assert result[2] == "c"
    print("✓ test_str_iteration_with_break passed")

def test_bytes_iteration_with_continue() -> None:
    """Test bytes iteration with continue statement"""
    b: bytes = b"hello"
    count: int = 0

    for byte_val in b:
        if byte_val == 108:  # 'l'
            continue
        count += 1

    assert count == 3  # 'h', 'e', 'o' (skipped two 'l's)
    print("✓ test_bytes_iteration_with_continue passed")

def test_nested_str_iteration() -> None:
    """Test nested string iteration"""
    s1: str = "ab"
    s2: str = "xy"
    result: list[str] = []

    for ch1 in s1:
        for ch2 in s2:
            result.append(ch1)
            result.append(ch2)

    assert len(result) == 8
    assert result[0] == "a"
    assert result[1] == "x"
    assert result[2] == "a"
    assert result[3] == "y"
    print("✓ test_nested_str_iteration passed")

def test_str_iteration_accumulate() -> None:
    """Test accumulating results from string iteration"""
    s: str = "12345"
    total: int = 0

    for ch in s:
        # Convert character to int (would be int(ch) in real Python)
        if ch == "1":
            total += 1
        elif ch == "2":
            total += 2
        elif ch == "3":
            total += 3
        elif ch == "4":
            total += 4
        elif ch == "5":
            total += 5

    assert total == 15
    print("✓ test_str_iteration_accumulate passed")

def test_bytes_bytearray_iteration_comparison() -> None:
    """Test that bytes and bytearray iterate the same way"""
    b: bytes = b"test"
    ba: bytearray = bytearray(b"test")

    bytes_result: list[int] = []
    for byte_val in b:
        bytes_result.append(byte_val)

    bytearray_result: list[int] = []
    for byte_val in ba:
        bytearray_result.append(byte_val)

    assert len(bytes_result) == len(bytearray_result)
    assert bytes_result[0] == bytearray_result[0]
    assert bytes_result[1] == bytearray_result[1]
    assert bytes_result[2] == bytearray_result[2]
    assert bytes_result[3] == bytearray_result[3]
    print("✓ test_bytes_bytearray_iteration_comparison passed")

def run_tests() -> None:
    test_str_iteration_basic()
    test_str_iteration_empty()
    test_str_iteration_single()
    test_bytes_iteration_basic()
    test_bytes_iteration_empty()
    test_bytes_iteration_values()
    test_bytearray_iteration_basic()
    test_bytearray_iteration_empty()
    test_bytearray_iteration_modified()
    test_str_iteration_with_break()
    test_bytes_iteration_with_continue()
    test_nested_str_iteration()
    test_str_iteration_accumulate()
    test_bytes_bytearray_iteration_comparison()
