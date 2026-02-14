def run_case() -> None:
    s: set[int] = {1, 2}
    h: int = s.__hash__(1)
    print(h)


if __name__ == "__main__":
    run_case()
