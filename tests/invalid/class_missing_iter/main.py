class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

f: Foo = Foo(1)
y: list[int] = [i for i in f]
