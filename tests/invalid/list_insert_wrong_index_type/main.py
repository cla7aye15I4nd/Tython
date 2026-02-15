def run_case() -> None:
    xs: list[int] = [1, 2, 3]
    xs.insert("bad", 42)


if __name__ == "__main__":
    run_case()
