def run_case() -> None:
    d: dict[int, int] = {1: 10}
    x: int = d.setdefault(2)

if __name__ == "__main__":
    run_case()
