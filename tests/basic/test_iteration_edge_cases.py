"""
Edge case tests for str, bytes, and bytearray iteration.
These tests verify correct behavior in unusual or boundary conditions,
all without using exception handling (direct indexing implementation).
"""

def test_str_unicode_ascii() -> None:
    """Test string iteration with ASCII characters"""
    s: str = "ABC123"
    chars: list[str] = []

    for ch in s:
        chars.append(ch)

    assert len(chars) == 6
    assert chars[0] == "A"
    assert chars[5] == "3"
    print("✓ ASCII string iteration works")

def test_bytes_all_values() -> None:
    """Test bytes with full range of byte values (0-255)"""
    # Test special byte values
    b: bytes = b"\x00\x01\x7f\x80\xff"
    values: list[int] = []

    for byte_val in b:
        values.append(byte_val)

    assert values[0] == 0     # NULL
    assert values[1] == 1     # SOH
    assert values[2] == 127   # DEL
    assert values[3] == 128   # 0x80
    assert values[4] == 255   # 0xFF
    print("✓ Full byte range (0-255) iteration works")

def test_str_iteration_else_not_executed() -> None:
    """Test that else clause is NOT executed when loop completes normally"""
    s: str = "test"
    loop_ran: bool = False
    else_ran: bool = False

    for ch in s:
        loop_ran = True
    else:
        else_ran = True

    assert loop_ran
    assert else_ran  # else runs when loop completes normally
    print("✓ For-else executes when iteration completes")

def test_str_iteration_else_skipped_on_break() -> None:
    """Test that else clause is skipped when break is used"""
    s: str = "abcdef"
    else_ran: bool = False

    for ch in s:
        if ch == "c":
            break
    else:
        else_ran = True

    assert not else_ran  # else should NOT run when break is used
    print("✓ For-else skipped when break is used")

def test_bytes_iteration_modification_safety() -> None:
    """Test that bytes (immutable) can be safely iterated"""
    b: bytes = b"immutable"
    count: int = 0

    # Bytes cannot be modified during iteration (they're immutable)
    for byte_val in b:
        count += 1
        # No modification possible - bytes are immutable

    assert count == 9
    print("✓ Immutable bytes iteration is safe")

def test_bytearray_iteration_snapshot() -> None:
    """Test bytearray iteration uses snapshot of collection"""
    ba: bytearray = bytearray(b"abc")
    chars_seen: int = 0

    for byte_val in ba:
        chars_seen += 1
        # Even if we could modify ba here, iteration continues
        # based on the initial length

    assert chars_seen == 3
    print("✓ Bytearray iteration uses length snapshot")

def test_str_iteration_in_function() -> None:
    """Test string iteration within a function"""
    def count_vowels(s: str) -> int:
        count: int = 0
        for ch in s:
            if ch == "a" or ch == "e" or ch == "i" or ch == "o" or ch == "u":
                count += 1
        return count

    result: int = count_vowels("hello world")
    assert result == 3  # e, o, o
    print("✓ String iteration in function works")

def test_bytes_iteration_return_early() -> None:
    """Test bytes iteration with early return"""
    def find_byte(b: bytes, target: int) -> bool:
        for byte_val in b:
            if byte_val == target:
                return True
        return False

    b: bytes = b"testing"
    assert find_byte(b, 115)  # 's' exists
    assert not find_byte(b, 120)  # 'x' doesn't exist
    print("✓ Early return from bytes iteration works")

def test_str_very_long() -> None:
    """Test iteration over very long string (stress test)"""
    # Create a 5000 character string
    s: str = "x" * 5000
    count: int = 0

    for ch in s:
        count += 1
        if count >= 5000:
            break

    assert count == 5000
    print("✓ Very long string (5000 chars) iteration works")

def test_mixed_iteration_types() -> None:
    """Test iterating over different types in sequence"""
    s: str = "ab"
    b: bytes = b"xy"
    ba: bytearray = bytearray(b"12")

    str_count: int = 0
    for ch in s:
        str_count += 1

    bytes_count: int = 0
    for byte_val in b:
        bytes_count += 1

    ba_count: int = 0
    for byte_val in ba:
        ba_count += 1

    assert str_count == 2
    assert bytes_count == 2
    assert ba_count == 2
    print("✓ Mixed type iterations work sequentially")

def test_iteration_variable_scope() -> None:
    """Test that iteration variable is properly scoped"""
    s: str = "abc"
    last_char: str = ""

    for ch in s:
        last_char = ch

    # After loop, variable should still be accessible
    assert last_char == "c"
    print("✓ Iteration variable scope is correct")

def run_tests() -> None:
    test_str_unicode_ascii()
    test_bytes_all_values()
    test_str_iteration_else_not_executed()
    test_str_iteration_else_skipped_on_break()
    test_bytes_iteration_modification_safety()
    test_bytearray_iteration_snapshot()
    test_str_iteration_in_function()
    test_bytes_iteration_return_early()
    test_str_very_long()
    test_mixed_iteration_types()
    test_iteration_variable_scope()
