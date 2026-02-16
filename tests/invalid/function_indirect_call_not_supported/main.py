def add(a: int, b: int) -> int:
    return a + b


def run_case() -> None:
    f = add
    x: int = f(1, 2)


if __name__ == "__main__":
    run_case()
