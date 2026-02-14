def run_case() -> None:
    b: bytearray = bytearray(b"a,b")
    out: list[bytearray] = b.split(1)
    print(out)


if __name__ == "__main__":
    run_case()
