class Boxy:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def run_case() -> None:
    -Boxy(1)

if __name__ == "__main__":
    run_case()
