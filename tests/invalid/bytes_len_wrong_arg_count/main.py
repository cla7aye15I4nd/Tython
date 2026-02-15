def run_case() -> None:
    b: bytes = b"hello"
    out: int = b.__len__(42)


if __name__ == "__main__":
    run_case()
