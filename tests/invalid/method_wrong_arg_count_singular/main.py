class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def add(self, n: int) -> None:
        self.x = self.x + n

f: Foo = Foo(1)
f.add(1, 2)
