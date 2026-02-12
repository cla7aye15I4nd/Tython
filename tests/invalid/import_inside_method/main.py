class C:
    x: int

    def __init__(self, x: int) -> None:
        import imports.module_a
        self.x = x


def main() -> None:
    c: C = C(1)
    print(c.x)
