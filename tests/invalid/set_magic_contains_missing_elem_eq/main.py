def run_case() -> None:
    a: set[set[int]] = set()
    x: set[int] = set()
    ok: bool = a.__contains__(x)
    print(ok)


if __name__ == "__main__":
    run_case()
