def run_case() -> None:
    s: set[int] = {1, 2}
    b: bool = s.__ne__()
    print(b)


if __name__ == "__main__":
    run_case()
