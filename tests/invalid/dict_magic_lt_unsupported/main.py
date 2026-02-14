def run_case() -> None:
    a: dict[int, int] = {1: 10}
    b: dict[int, int] = {2: 20}
    out: bool = a.__lt__(b)


if __name__ == "__main__":
    run_case()
