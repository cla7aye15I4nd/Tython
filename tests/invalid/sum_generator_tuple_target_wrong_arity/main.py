def run_case() -> None:
    total: int = sum((a for (i, a, z) in enumerate([1, 2, 3])), 0)
    print(total)


if __name__ == "__main__":
    run_case()
