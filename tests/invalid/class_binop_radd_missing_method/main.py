class RightOnly:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def run_case() -> None:
    bad: int = 1 + RightOnly(2)
    print(bad)


if __name__ == "__main__":
    run_case()
