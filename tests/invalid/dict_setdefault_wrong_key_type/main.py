def run_case() -> None:
    d: dict[int, int] = {1: 10}
    x: int = d.setdefault("k", 3)

if __name__ == "__main__":
    run_case()
