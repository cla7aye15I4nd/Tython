def run_case() -> None:
    s: set[int] = {1, 2}
    out: set[int] = s.__iand__(1)
    print(out)


if __name__ == "__main__":
    run_case()
