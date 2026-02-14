"""
Performance test demonstrating efficient iteration without exceptions.
This test shows that large strings/bytes can be iterated efficiently
using direct indexing instead of the exception-based __iter__/__next__ protocol.
"""

def test_large_string_iteration() -> None:
    """Test iteration over a large string (no exception overhead)"""
    # Create a string with 1000 characters
    s: str = "a" * 1000

    count: int = 0
    for ch in s:
        count += 1

    assert count == 1000
    print("✓ Iterated over 1000-character string efficiently")

def test_large_bytes_iteration() -> None:
    """Test iteration over large bytes (no exception overhead)"""
    # Create bytes with 1000 elements
    b: bytes = b"x" * 1000

    count: int = 0
    sum_values: int = 0
    for byte_val in b:
        count += 1
        sum_values += byte_val

    assert count == 1000
    assert sum_values == 120000  # 1000 * 120 (ord('x'))
    print("✓ Iterated over 1000-byte bytes object efficiently")

def test_large_bytearray_iteration() -> None:
    """Test iteration over large bytearray (no exception overhead)"""
    # Create bytearray with 500 elements (all zeros)
    ba: bytearray = bytearray(500)

    count: int = 0
    for byte_val in ba:
        count += 1

    assert count == 500
    print("✓ Iterated over 500-element bytearray efficiently")

def test_iteration_with_complex_logic() -> None:
    """Test iteration with complex logic in the loop body"""
    s: str = "abcdefghijklmnopqrstuvwxyz" * 10  # 260 characters

    vowel_count: int = 0
    consonant_count: int = 0

    for ch in s:
        if ch == "a" or ch == "e" or ch == "i" or ch == "o" or ch == "u":
            vowel_count += 1
        else:
            consonant_count += 1

    assert vowel_count == 50   # 5 vowels * 10 repetitions
    assert consonant_count == 210  # 21 consonants * 10 repetitions
    print("✓ Iterated with complex logic efficiently")

def test_multiple_sequential_iterations() -> None:
    """Test that we can iterate the same collection multiple times"""
    s: str = "test"

    # First iteration
    count1: int = 0
    for ch in s:
        count1 += 1

    # Second iteration
    count2: int = 0
    for ch in s:
        count2 += 1

    # Third iteration
    count3: int = 0
    for ch in s:
        count3 += 1

    assert count1 == 4
    assert count2 == 4
    assert count3 == 4
    print("✓ Same collection iterated multiple times efficiently")

def test_bytes_iteration_with_filtering() -> None:
    """Test filtering during bytes iteration"""
    # Create bytes with repeating pattern
    b: bytes = b"abcdefghijklmnopqrstuvwxyz" * 10  # 260 bytes

    # Count bytes with value > 100 (ASCII 'e' onwards in lowercase)
    count: int = 0
    for byte_val in b:
        if byte_val > 100:  # ASCII values > 100 (e, f, g, ..., z)
            count += 1

    assert count == 220  # 22 letters from 'e' to 'z' * 10 repetitions
    print("✓ Filtered bytes iteration efficiently")

def run_tests() -> None:
    test_large_string_iteration()
    test_large_bytes_iteration()
    test_large_bytearray_iteration()
    test_iteration_with_complex_logic()
    test_multiple_sequential_iterations()
    test_bytes_iteration_with_filtering()
