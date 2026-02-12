class Foo:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __iter__(self) -> int:
        return 0

f: Foo = Foo(1)
for i in f:
    print(i)
