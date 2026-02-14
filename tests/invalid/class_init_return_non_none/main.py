class BadInit:
    value: int

    def __init__(self, value: int) -> int:
        self.value = value
        return value


def run_case() -> None:
    BadInit(1)


if __name__ == "__main__":
    run_case()
