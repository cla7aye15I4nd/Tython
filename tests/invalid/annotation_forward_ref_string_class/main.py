class C:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n


def run_case() -> None:
    bad: "C" = 1


if __name__ == "__main__":
    run_case()
