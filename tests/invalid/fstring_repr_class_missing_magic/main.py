class NoRepr:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    s: str = f"{NoRepr(2)!r}"
    print(s)


if __name__ == "__main__":
    main()
