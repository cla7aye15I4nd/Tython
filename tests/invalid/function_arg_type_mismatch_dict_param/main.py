def pick(m: dict[int, int]) -> int:
    return m[1]


def run_case() -> None:
    bad: int = pick([1, 2])
    print(bad)


if __name__ == "__main__":
    run_case()
