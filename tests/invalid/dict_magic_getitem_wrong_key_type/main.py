def run_case() -> None:
    d: dict[int, int] = {1: 10}
    v: int = d.__getitem__("1")


if __name__ == "__main__":
    run_case()
