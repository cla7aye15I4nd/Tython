def run_case() -> None:
    b: bytes = b"hello"
    out: str = b.__str__(42)


if __name__ == "__main__":
    run_case()
