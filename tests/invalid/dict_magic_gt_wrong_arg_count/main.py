def run_case() -> None:
    a: dict[int, int] = {1: 10}
    out: bool = a.__gt__()


if __name__ == "__main__":
    run_case()
