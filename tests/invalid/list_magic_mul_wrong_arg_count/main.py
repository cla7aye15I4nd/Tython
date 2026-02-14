def run_case() -> None:
    xs: list[int] = [1, 2]
    ys: list[int] = xs.__mul__()
    print(ys)


if __name__ == "__main__":
    run_case()
