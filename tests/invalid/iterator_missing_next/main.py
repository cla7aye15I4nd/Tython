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

f: Foo = Foo(1)
y: list[int] = [i for i in f]
