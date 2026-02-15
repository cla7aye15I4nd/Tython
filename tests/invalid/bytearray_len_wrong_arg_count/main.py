def run_case() -> None:
    ba: bytearray = bytearray(b"hello")
    out: int = ba.__len__(42)


if __name__ == "__main__":
    run_case()
