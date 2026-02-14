class BoxNoEq:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    d: dict[BoxNoEq, int] = {}
    d[BoxNoEq(1)] = 7


if __name__ == "__main__":
    main()
