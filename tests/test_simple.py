"""
Test simple functions with no imports.
"""

def factorial(n: int) -> int:
    """Calculate factorial of n."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def add(a: int, b: int) -> int:
    """Simple addition."""
    return a + b
