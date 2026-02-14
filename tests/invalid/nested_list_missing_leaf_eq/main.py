class BoxNoEq:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    xs: list[list[BoxNoEq]] = [[BoxNoEq(1)], [BoxNoEq(2)]]
    count: int = xs.count([BoxNoEq(1)])
    print(count)


if __name__ == "__main__":
    main()
