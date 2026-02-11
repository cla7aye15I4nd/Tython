class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __str__(self, n: int) -> str:
        return "x"

f: Foo = Foo(1)
s: str = str(f)
