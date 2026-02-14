def run_case() -> None:
    xs: list[int] = [0, 1, 2, 3]
    ys: list[int] = xs["a":3]
    print(ys)


if __name__ == "__main__":
    run_case()
