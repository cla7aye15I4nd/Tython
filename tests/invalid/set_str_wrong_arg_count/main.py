def run_case() -> None:
    s: set[int] = {1, 2}
    out: str = s.__str__(42)


if __name__ == "__main__":
    run_case()
