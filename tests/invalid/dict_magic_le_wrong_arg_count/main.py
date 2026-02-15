def run_case() -> None:
    a: dict[int, int] = {1: 10}
    out: bool = a.__le__()


if __name__ == "__main__":
    run_case()
