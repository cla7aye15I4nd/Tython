class Box:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n


def run_case() -> None:
    a: Box = Box(1)
    b: Box = Box(2)
    ok: bool = a >= b


if __name__ == "__main__":
    run_case()
