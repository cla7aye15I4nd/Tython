class Outer:
    base: int

    class Inner:
        value: int

        def __init__(self, value: int) -> None:
            self.value = value

        def get(self) -> int:
            return self.value

    def __init__(self, base: int) -> None:
        self.base = base

    def get_base(self) -> int:
        return self.base


class Deep:
    class Mid:
        class Leaf:
            n: int

            def __init__(self, n: int) -> None:
                self.n = n

            def triple(self) -> int:
                return self.n * 3

