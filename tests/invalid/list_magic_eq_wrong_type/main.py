def run_case() -> None:
    xs: list[int] = [1, 2]
    ok: bool = xs.__eq__(123)
    print(ok)


if __name__ == "__main__":
    run_case()
