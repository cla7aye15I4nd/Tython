def run_case() -> None:
    s: set[int] = {1, 2}
    s.__rsub__()


if __name__ == "__main__":
    run_case()
