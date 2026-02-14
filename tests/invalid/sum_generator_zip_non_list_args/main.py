def run_case() -> None:
    total: int = sum((a + b for (a, b) in zip(1, [2, 3])), 0)
    print(total)


if __name__ == "__main__":
    run_case()
