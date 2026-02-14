class Box:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

def run_case() -> None:
    xs: list[Box] = [Box(3), Box(1), Box(2)]

    ys: list[Box] = sorted(xs)

if __name__ == "__main__":
    run_case()
