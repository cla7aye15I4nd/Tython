def run_case() -> None:
    d: dict[int, int] = {1: 10}
    out: bool = d.__eq__(42)


if __name__ == "__main__":
    run_case()
