class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

f: Foo = Foo(1)
f.x = 1.0
