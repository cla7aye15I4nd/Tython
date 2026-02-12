def main() -> None:
    def inner(x) -> int:
        return x

    y: int = inner(1)
    assert y == 1
