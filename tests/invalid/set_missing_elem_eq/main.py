class BoxNoEq:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    s: set[BoxNoEq] = set()
    s.add(BoxNoEq(1))


if __name__ == "__main__":
    main()
