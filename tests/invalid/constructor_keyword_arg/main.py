class Point:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x

def run_case() -> None:
    p: Point = Point(x=1)

if __name__ == "__main__":
    run_case()
