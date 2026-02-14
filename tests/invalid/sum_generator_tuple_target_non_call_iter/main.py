def run_case() -> None:
    pairs: list[tuple[int, int]] = [(1, 2), (3, 4)]
    total: int = sum((a + b for a, b in pairs), 0)
    print(total)


if __name__ == "__main__":
    run_case()
