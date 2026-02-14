def run_case() -> None:
    b: bytes = b"a,b"
    out: list[bytes] = b.split(1)
    print(out)


if __name__ == "__main__":
    run_case()
