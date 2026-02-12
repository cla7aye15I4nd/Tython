def main() -> None:
    a: tuple[int, int] = (1, 2)
    b: tuple[int, float] = (1, 2.0)
    x: bool = a == b
    print(x)
