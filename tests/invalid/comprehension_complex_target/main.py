def run_case() -> None:
    pairs: list[tuple[int, int]] = [(1, 2), (3, 4)]

    x: list[int] = [a for (a, b) in pairs]

if __name__ == "__main__":
    run_case()
