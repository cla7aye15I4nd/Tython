class C:
    def __iter__(self) -> "C":
        return self
    def __next__(self) -> int:
        return 1

def run_case() -> None:
    x: int = next(C(), 0)

if __name__ == "__main__":
    run_case()
