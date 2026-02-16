class A:
    pass


def main() -> None:
    x: A = A()
    y: A = A()
    _ = x // y
