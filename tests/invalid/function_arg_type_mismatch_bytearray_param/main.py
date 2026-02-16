def takes_bytearray(x: bytearray) -> int:
    return len(x)


def run_case() -> None:
    y: int = takes_bytearray(1)


if __name__ == "__main__":
    run_case()
