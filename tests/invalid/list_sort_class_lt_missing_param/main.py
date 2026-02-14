class Box:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

    def __lt__(self) -> bool:
        return True


def run_case() -> None:
    xs: list[Box] = [Box(2), Box(1)]
    xs.sort()


if __name__ == "__main__":
    run_case()
