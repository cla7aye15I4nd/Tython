def run_case() -> None:
    pairs: list[tuple[int, int]] = [(1, 2), (3, 4)]
    out: list[int] = [a + b for a, b in pairs]
    print(out)


if __name__ == "__main__":
    run_case()
