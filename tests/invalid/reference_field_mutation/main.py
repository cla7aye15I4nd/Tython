class Inner:
    val: int

    def __init__(self, val: int) -> None:
        self.val = val

class Outer:
    inner: Inner

    def __init__(self, inner: Inner) -> None:
        self.inner = inner

    def replace(self, new_inner: Inner) -> None:
        self.inner = new_inner
