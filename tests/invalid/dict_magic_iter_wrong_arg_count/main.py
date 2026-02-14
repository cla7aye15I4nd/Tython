def run_case() -> None:
    d: dict[int, int] = {1: 10}
    d.__iter__(1)


if __name__ == "__main__":
    run_case()
