def run_case() -> None:
    t: tuple[int, int, int] = (1, 2, 3)
    x: int = t[True]
    print(x)


if __name__ == "__main__":
    run_case()
