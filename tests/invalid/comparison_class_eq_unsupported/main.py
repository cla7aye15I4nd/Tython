class Ghost:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def run_case() -> None:
    a: Ghost = Ghost(1)
    b: Ghost = Ghost(2)
    _ = a < b

if __name__ == "__main__":
    run_case()
