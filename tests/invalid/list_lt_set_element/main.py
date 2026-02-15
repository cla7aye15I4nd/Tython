def run_case() -> None:
    a: list[set[int]] = [{1, 2}]
    b: list[set[int]] = [{3, 4}]
    out: bool = a.__lt__(b)


if __name__ == "__main__":
    run_case()
