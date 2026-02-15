def run_case() -> None:
    xs: list[int] = [1, 2]
    out: list[int] = xs.__rmul__("bad")


if __name__ == "__main__":
    run_case()
