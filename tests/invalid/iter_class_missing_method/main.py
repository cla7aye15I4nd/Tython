class C:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x


def main() -> None:
    it = iter(C(1))
    print(it)


if __name__ == "__main__":
    main()
