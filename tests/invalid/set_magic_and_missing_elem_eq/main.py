def run_case() -> None:
    a: set[set[int]] = set()
    b: set[set[int]] = set()
    out: set[set[int]] = a.__and__(b)
    print(out)


if __name__ == "__main__":
    run_case()
