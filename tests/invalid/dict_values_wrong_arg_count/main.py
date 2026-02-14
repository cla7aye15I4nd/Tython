def run_case() -> None:
    d: dict[int, int] = {1: 2}
    out: list[int] = d.values(1)
    print(out)


if __name__ == "__main__":
    run_case()
