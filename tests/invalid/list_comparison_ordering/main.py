class Box:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

def run_case() -> None:
    a: list[Box] = [Box(1), Box(2)]

    b: list[Box] = [Box(3), Box(4)]

    c: bool = a < b

if __name__ == "__main__":
    run_case()
