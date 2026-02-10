class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def get_x(self) -> int:
        return self.x

f: Foo = Foo(1)
f.get_x(42)
