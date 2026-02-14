class Outer:
    class Inner:
        n: int

        def __init__(self, n: int) -> None:
            self.n = n


def run_case() -> None:
    bad: Outer.Inner = 1


if __name__ == "__main__":
    run_case()
