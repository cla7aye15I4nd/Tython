def run_case() -> None:
    d: dict[int, int] = {1: 10}
    d.__setitem__(2, "bad")


if __name__ == "__main__":
    run_case()
