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


def run_case() -> None:
    outer: Outer = Outer(Inner(7))
    next_inner: Inner = Inner(9)
    outer.replace(next_inner)


if __name__ == "__main__":
    run_case()
