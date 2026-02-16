class C:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x


def main() -> None:
    out: bytes = bytes(C(1))
    print(out)


if __name__ == "__main__":
    main()
