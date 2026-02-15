def run_case() -> None:
    ba: bytearray = bytearray(b"hello")
    out: bool = ba.__lt__()


if __name__ == "__main__":
    run_case()
