def run_case() -> None:
    d: dict[int, int] = {1: 2}
    v: int = d.__getitem__()
    print(v)


if __name__ == "__main__":
    run_case()
