class C:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n


def main() -> None:
    a: C = C(1)
    b: C = C(2)
    x: bool = a < b
