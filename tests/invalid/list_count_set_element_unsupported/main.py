def run_case() -> None:
    xs: list[set[int]] = [{1}, {2}]
    c: int = xs.count({1})
    print(c)


if __name__ == "__main__":
    run_case()
