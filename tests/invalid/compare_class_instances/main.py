class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

a: Foo = Foo(1)
b: Foo = Foo(2)
c: bool = a == b
