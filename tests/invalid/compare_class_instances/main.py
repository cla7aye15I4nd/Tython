class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

def run_case() -> None:
    a: Foo = Foo(1)

    b: Foo = Foo(2)

    c: bool = a < b

if __name__ == "__main__":
    run_case()
