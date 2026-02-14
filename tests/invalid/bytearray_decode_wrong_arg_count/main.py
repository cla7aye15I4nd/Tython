def run_case() -> None:
    b: bytearray = bytearray(b"abc")
    out: str = b.decode(1)
    print(out)


if __name__ == "__main__":
    run_case()
