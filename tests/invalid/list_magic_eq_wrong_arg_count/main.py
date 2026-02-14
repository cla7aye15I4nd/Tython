def run_case() -> None:
    xs: list[int] = [1, 2]
    b: bool = xs.__eq__()
    print(b)


if __name__ == "__main__":
    run_case()
