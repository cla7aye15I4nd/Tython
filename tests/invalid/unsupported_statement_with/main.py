class Ctx:
    x: int
    def __init__(self) -> None:
        self.x = 0

def main() -> None:
    with Ctx() as c:
        print(c)
