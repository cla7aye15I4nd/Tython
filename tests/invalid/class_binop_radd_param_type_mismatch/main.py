class RightAdder:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __radd__(self, other: int) -> int:
        return other + self.value


def run_case() -> None:
    bad: int = 1.5 + RightAdder(2)
    print(bad)


if __name__ == "__main__":
    run_case()
