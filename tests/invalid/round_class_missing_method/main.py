class C:
    x: int

    def __init__(self, x: int) -> None:
        self.x = x


def main() -> None:
    out: int = round(C(1))
    print(out)


if __name__ == "__main__":
    main()
