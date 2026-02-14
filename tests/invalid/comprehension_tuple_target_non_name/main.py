def run_case() -> None:
    x: list[int] = [a for (a, (b, c)) in zip([1], [(2, 3)])]

if __name__ == "__main__":
    run_case()
