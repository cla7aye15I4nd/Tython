class FooIter:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x
class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __iter__(self) -> FooIter:
        return FooIter(self.x)

def run_case() -> None:
    f: Foo = Foo(1)

    for i in f:
        print(i)

if __name__ == "__main__":
    run_case()
