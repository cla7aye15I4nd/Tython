def run_case() -> None:
    s: set[int] = {1, 2}
    ok: bool = s.isdisjoint(1)
    print(ok)


if __name__ == "__main__":
    run_case()
