class Box:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __add__(self, other: "Box") -> "Box":
        return Box(self.n + other.n)


def run_case() -> None:
    a: Box = Box(1)
    bad: Box = a + 2
    print(bad.n)


if __name__ == "__main__":
    run_case()
