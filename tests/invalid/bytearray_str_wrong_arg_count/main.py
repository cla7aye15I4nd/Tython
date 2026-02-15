def run_case() -> None:
    ba: bytearray = bytearray(b"hello")
    out: str = ba.__str__(42)


if __name__ == "__main__":
    run_case()
