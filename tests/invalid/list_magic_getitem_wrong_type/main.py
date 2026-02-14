def run_case() -> None:
    xs: list[int] = [2, 1]
    i: int = xs.__getitem__("0")
    print(i)


if __name__ == "__main__":
    run_case()
