def run_case() -> None:
    s: set[int] = {1, 2}
    ok: bool = s.__contains__("x")
    print(ok)


if __name__ == "__main__":
    run_case()
