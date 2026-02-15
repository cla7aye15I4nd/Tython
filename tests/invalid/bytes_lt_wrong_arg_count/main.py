def run_case() -> None:
    b: bytes = b"hello"
    out: bool = b.__lt__()


if __name__ == "__main__":
    run_case()
