class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __iter__(self) -> int:
        return 0

def run_case() -> None:
    f: Foo = Foo(1)

    y: list[int] = [i for i in f]

if __name__ == "__main__":
    run_case()
