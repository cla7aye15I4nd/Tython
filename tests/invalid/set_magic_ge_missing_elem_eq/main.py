def run_case() -> None:
    a: set[set[int]] = set()
    b: set[set[int]] = set()
    ok: bool = a.__ge__(b)
    print(ok)


if __name__ == "__main__":
    run_case()
