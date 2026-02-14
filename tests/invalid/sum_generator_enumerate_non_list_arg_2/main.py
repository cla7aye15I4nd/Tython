def run_case() -> None:
    total: int = sum((i + v for (i, v) in enumerate(123)), 0)
    print(total)


if __name__ == "__main__":
    run_case()
