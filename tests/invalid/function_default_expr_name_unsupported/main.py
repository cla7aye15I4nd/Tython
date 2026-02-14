def base() -> int:
    return 3


def f(x: int = base) -> int:
    return x


def run_case() -> None:
    print(f())


if __name__ == "__main__":
    run_case()
