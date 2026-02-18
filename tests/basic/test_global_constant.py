PI: float = 3.0
RADIUS: float = 4.0
TWO: int = 1 + 1
GREETING: str = "hello"
ENABLED: bool = True
BYTES_LABEL: bytes = b"ok"


def circle_area() -> float:
    return PI * RADIUS * RADIUS


def read_later() -> int:
    return LATER


LATER: int = 7
VALUES: list[int] = [1, 2, 3]


def run_tests() -> None:
    area: float = circle_area()
    print(int(area))
    print(TWO)
    print(GREETING)
    print(ENABLED)
    print(BYTES_LABEL)
    print(read_later())
    print(len(VALUES))
    print(VALUES[0])
    print(VALUES[2])


if __name__ == "__main__":
    run_tests()
