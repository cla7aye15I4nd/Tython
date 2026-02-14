class KeyEq:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __eq__(self, other: "KeyEq") -> bool:
        return self.value == other.value


class ValueNoEq:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value


def main() -> None:
    a: dict[KeyEq, ValueNoEq] = {KeyEq(1): ValueNoEq(7)}
    b: dict[KeyEq, ValueNoEq] = {KeyEq(1): ValueNoEq(7)}
    print(a == b)


if __name__ == "__main__":
    main()
