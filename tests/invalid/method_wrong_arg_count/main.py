class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def get_x(self) -> int:
        return self.x

def run_case() -> None:
    f: Foo = Foo(1)

    f.get_x(42)

if __name__ == "__main__":
    run_case()
