def run_case() -> None:
    d: dict[int, int] = {1: 2}
    d.__delitem__()


if __name__ == "__main__":
    run_case()
