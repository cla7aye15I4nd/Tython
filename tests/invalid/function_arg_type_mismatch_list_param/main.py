def head(xs: list[int]) -> int:
    return xs[0]


def run_case() -> None:
    bad: int = head((1, 2))
    print(bad)


if __name__ == "__main__":
    run_case()
