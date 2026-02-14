class Pair:
    x: int
    y: int

    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def run_case() -> None:
    p: Pair = Pair(1)

if __name__ == "__main__":
    run_case()
