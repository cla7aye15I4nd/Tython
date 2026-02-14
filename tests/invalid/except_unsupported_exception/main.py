def run_case() -> None:
    try:
        x: int = 1
    except FooError:
        x: int = 2

if __name__ == "__main__":
    run_case()
