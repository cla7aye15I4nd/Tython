def first_pair(p: tuple[int, int]) -> int:
    return p[0]


def run_case() -> None:
    bad: int = first_pair([1, 2])
    print(bad)


if __name__ == "__main__":
    run_case()
