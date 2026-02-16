class BadRepr:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __repr__(self) -> None:
        pass


def run_case() -> None:
    s: str = f"{BadRepr(1)!r}"
    print(s)


if __name__ == "__main__":
    run_case()
