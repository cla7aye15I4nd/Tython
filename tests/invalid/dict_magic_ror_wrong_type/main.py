def run_case() -> None:
    d: dict[int, int] = {1: 10}
    out: dict[int, int] = d.__ror__(1)
    print(out)


if __name__ == "__main__":
    run_case()
