def run_case() -> None:
    d: dict[int, int] = {1: 2}
    b: bool = d.__eq__()
    print(b)


if __name__ == "__main__":
    run_case()
