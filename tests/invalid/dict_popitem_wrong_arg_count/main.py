def run_case() -> None:
    d: dict[int, int] = {1: 2}
    out: tuple[int, int] = d.popitem(1)
    print(out)


if __name__ == "__main__":
    run_case()
