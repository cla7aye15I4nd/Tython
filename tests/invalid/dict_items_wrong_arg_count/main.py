def run_case() -> None:
    d: dict[int, int] = {1: 2}
    out: list[tuple[int, int]] = d.items(1)
    print(out)


if __name__ == "__main__":
    run_case()
