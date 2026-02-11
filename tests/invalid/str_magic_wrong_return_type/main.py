class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __str__(self) -> int:
        return self.x

f: Foo = Foo(1)
s: str = str(f)
