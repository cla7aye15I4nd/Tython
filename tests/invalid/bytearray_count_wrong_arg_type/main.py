def run_case() -> None:
    ba: bytearray = bytearray(b"ababa")
    n: int = ba.count(1)
    print(n)


if __name__ == "__main__":
    run_case()
