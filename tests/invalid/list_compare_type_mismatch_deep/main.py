def main() -> None:
    a: list[list[int]] = [[1], [2]]
    b: list[list[float]] = [[1.0], [2.0]]
    x: bool = a == b
    print(x)
