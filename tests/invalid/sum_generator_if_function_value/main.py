def helper(x: int) -> int:
    return x


def run_case() -> None:
    total: int = sum((i for i in [1, 2, 3] if helper), 0)
    print(total)


if __name__ == "__main__":
    run_case()
