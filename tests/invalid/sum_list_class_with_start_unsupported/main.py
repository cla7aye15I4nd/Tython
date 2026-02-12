class Box:
    v: int

    def __init__(self, v: int) -> None:
        self.v = v


def main() -> None:
    xs: list[Box] = [Box(1), Box(2)]
    y = sum(xs, 0)
    print(y)
