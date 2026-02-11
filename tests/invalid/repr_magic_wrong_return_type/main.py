class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __repr__(self) -> int:
        return self.x

f: Foo = Foo(1)
s: str = repr(f)
