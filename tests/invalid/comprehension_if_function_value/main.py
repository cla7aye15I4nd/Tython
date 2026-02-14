def helper(x: int) -> int:
    return x


def run_case() -> None:
    out: list[int] = [i for i in [1, 2, 3] if helper]
    print(out)


if __name__ == "__main__":
    run_case()
