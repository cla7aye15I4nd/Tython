class Box:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __add__(self) -> "Box":
        return Box(self.n)


def run_case() -> None:
    a: Box = Box(1)
    b: Box = Box(2)
    bad: Box = a + b
    print(bad.n)


if __name__ == "__main__":
    run_case()
