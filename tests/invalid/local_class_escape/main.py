def make() -> int:
    class Secret:
        x: int

        def __init__(self, x: int) -> None:
            self.x = x

    s: Secret = Secret(1)
    return s.x


def main() -> None:
    s: Secret = Secret(42)
