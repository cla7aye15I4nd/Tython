class Foo:
    x: int
    def __init__(self) -> None:
        self.x = 10
    def bad(self) -> None:
        self.x /= 3
