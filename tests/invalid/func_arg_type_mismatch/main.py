def add(a: int, b: int) -> int:
    return a + b

def run_case() -> None:
    x: int = add(1, 2.0)

if __name__ == "__main__":
    run_case()
