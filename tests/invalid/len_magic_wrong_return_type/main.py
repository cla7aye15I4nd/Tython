class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __len__(self) -> str:
        return "x"

def run_case() -> None:
    f: Foo = Foo(1)

    n: int = len(f)

if __name__ == "__main__":
    run_case()
