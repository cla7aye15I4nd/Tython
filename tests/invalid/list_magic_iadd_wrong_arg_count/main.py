def run_case() -> None:
    xs: list[int] = [1, 2]
    ys: list[int] = [3]
    out: list[int] = xs.__iadd__()
    print(out)


if __name__ == "__main__":
    run_case()
