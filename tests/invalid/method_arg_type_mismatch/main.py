class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def set_x(self, x: int) -> None:
        self.x = x

f: Foo = Foo(1)
f.set_x(1.0)
