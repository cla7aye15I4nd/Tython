class BoxNoLt:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    xs: list[list[BoxNoLt]] = [[BoxNoLt(2)], [BoxNoLt(1)]]
    xs.sort()


if __name__ == "__main__":
    main()
