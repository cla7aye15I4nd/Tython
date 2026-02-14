def run_case() -> None:
    xs: list[int] = [1, 2]
    out: list[int] = xs.__imul__()
    print(out)


if __name__ == "__main__":
    run_case()
