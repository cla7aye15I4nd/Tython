def run_case() -> None:
    s: set[int] = {1, 2}
    out: list[int] = s.__iter__(1)
    print(out)


if __name__ == "__main__":
    run_case()
