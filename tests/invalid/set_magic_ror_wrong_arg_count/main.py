def run_case() -> None:
    s: set[int] = {1, 2}
    s.__ror__()


if __name__ == "__main__":
    run_case()
