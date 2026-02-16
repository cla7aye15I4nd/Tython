class Box:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x


def run_case() -> None:
    box: Box = Box(x=1)


if __name__ == '__main__':
    run_case()
