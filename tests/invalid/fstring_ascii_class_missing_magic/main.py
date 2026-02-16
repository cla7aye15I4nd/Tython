class NoAscii:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def run_case() -> None:
    s: str = f"{NoAscii(1)!a}"
    print(s)


if __name__ == "__main__":
    run_case()
