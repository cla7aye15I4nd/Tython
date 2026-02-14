def run_case() -> None:
    xs: list[int] = [1, 2]
    ys: list[int] = xs.__reversed__(1)
    print(ys)


if __name__ == "__main__":
    run_case()
