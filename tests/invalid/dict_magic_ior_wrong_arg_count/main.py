def run_case() -> None:
    d: dict[int, int] = {1: 2}
    out: dict[int, int] = d.__ior__()
    print(out)


if __name__ == "__main__":
    run_case()
