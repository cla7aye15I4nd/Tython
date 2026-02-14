class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

def run_case() -> None:
    f: Foo = Foo(1)

    f.nonexistent()

if __name__ == "__main__":
    run_case()
