class NoStr:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    s: str = f"{NoStr(1)!s}"
    print(s)


if __name__ == "__main__":
    main()
