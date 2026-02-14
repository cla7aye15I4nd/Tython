def run_case() -> None:
    x: int = 1

    def inner() -> int:
        return x

    x = "oops"
    y: int = inner()


if __name__ == "__main__":
    run_case()
