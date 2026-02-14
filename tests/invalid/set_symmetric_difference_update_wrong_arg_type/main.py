def run_case() -> None:
    s: set[int] = {1, 2}
    s.symmetric_difference_update(1)


if __name__ == "__main__":
    run_case()
