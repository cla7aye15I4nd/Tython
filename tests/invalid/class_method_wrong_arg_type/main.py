class Counter:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def add(self, x: int) -> None:
        self.value = self.value + x


def run_case() -> None:
    c: Counter = Counter(1)
    c.add("x")


if __name__ == "__main__":
    run_case()
