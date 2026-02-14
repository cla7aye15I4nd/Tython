def run_case() -> None:
    xs: list[int] = [1, 2]
    v: int = xs.__getitem__()
    print(v)


if __name__ == "__main__":
    run_case()
