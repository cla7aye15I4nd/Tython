class Box:
    def __getitem__(self, index: int) -> None:
        return None


def main() -> None:
    b: Box = Box()
    x: int = b[0]
