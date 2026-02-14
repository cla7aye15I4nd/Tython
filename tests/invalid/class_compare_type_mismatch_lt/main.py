class Left:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __lt__(self, other: "Left") -> bool:
        return self.n < other.n


class Right:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n


def run_case() -> None:
    a: Left = Left(1)
    b: Right = Right(2)
    ok: bool = a < b


if __name__ == "__main__":
    run_case()
