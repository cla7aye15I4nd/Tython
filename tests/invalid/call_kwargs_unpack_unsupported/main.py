def add(a: int, b: int) -> int:
    return a + b

if __name__ == "__main__":
    x: int = add(**{"a": 1, "b": 2})
