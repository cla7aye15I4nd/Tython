def run_case() -> None:
    xs: list[int] = [1, 2]
    xs.extend("bad")


if __name__ == "__main__":
    run_case()
