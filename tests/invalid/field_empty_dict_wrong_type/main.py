class Foo:
    x: int

    def __init__(self) -> None:
        self.x = {}


def run_case() -> None:
    Foo()


if __name__ == "__main__":
    run_case()
