def run_case() -> None:
    d: dict[int, int] = {1: 10}
    d.__delitem__("bad")


if __name__ == "__main__":
    run_case()
